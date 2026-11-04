//! History disclosure is decided by native roots, independently of projection code.
use crate::domain_computation::primary_graph::tests::application_query::*;

#[test]
fn correction_disclosure_requires_the_original_native_root_after_filtering() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let original_subject = account.entity_id_for_test();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let query = installed_query(&world);
            for (status, expected) in [("open", true), ("no matching account", false)] {
                let plan = world
                    .selected_product(&phase.test_host_request_for_test())
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), status.to_owned())
                            .unwrap(),
                        current_controls(&request),
                    )
                    .unwrap();
                let execution = phase
                    .execution_request_for_for_test(&(*(*world.application.runtime_for_test()).product_runtime_for_test()))
                    .unwrap();
                assert_eq!(
                    world
                        .application
                        .execute_correction_disclosure_for_test(&execution, plan, original_subject)
                        .unwrap(),
                    expected
                );
            }
            // This query reads the account's activities. An authorized, nonempty read
            // under the correct scope is insufficient when that scope is not a root.
            let related = world
                .application
                .installed_schema()
                .certification_query(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CrossRootQuery::reference())
                .unwrap();
            let ordinary_plan = world
                .selected_product(&phase.test_host_request_for_test())
                .admit_application_query(
                    &phase,
                    &related,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap();
            let ordinary = world
                .application
                .execute_application_query_one_shot(&phase, ordinary_plan)
                .unwrap();
            assert!(
                !ordinary.rows().is_empty(),
                "the related-root refusal must distinguish a real nonempty read"
            );
            let plan = world
                .selected_product(&phase.test_host_request_for_test())
                .admit_application_query(
                    &phase,
                    &related,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap();
            let execution = phase
                .execution_request_for_for_test(&(*(*world.application.runtime_for_test()).product_runtime_for_test()))
                .unwrap();
            assert!(!world
                .application
                .execute_correction_disclosure_for_test(&execution, plan, original_subject)
                .unwrap());
        })
        .unwrap();
}

#[test]
fn correction_disclosure_never_calls_the_application_projector() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world.application.with_host_advancement(|phase| {
        let selected = world.selected_product(&phase.test_host_request_for_test());
        let principal = selected.resolve_authenticated_principal(
            &phase, &world.binding, &external, &request, WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
        let account = selected.resolve_entity(
            &phase, AccountStatus::reference(), "open".to_owned(), &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
        let subject = account.entity_id_for_test();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
        let query = installed_forged_selector_query(&world);
        let plan = world.selected_product(&phase.test_host_request_for_test()).admit_application_query(
            &phase, &query, &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_owned()).unwrap(),
            current_controls(&request),
        ).unwrap();
        let execution = phase.execution_request_for_for_test(&(*(*world.application.runtime_for_test()).product_runtime_for_test())).unwrap();
        assert!(world.application.execute_correction_disclosure_for_test(&execution, plan, subject).unwrap());
        // The unchanged ordinary projection is a distinguishing counterpart: its
        // forged field selector fails, while the native root above remains real.
        let plan = world.selected_product(&phase.test_host_request_for_test()).admit_application_query(
            &phase, &query, &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_owned()).unwrap(),
            current_controls(&request),
        ).unwrap();
        assert!(matches!(world.application.execute_application_query_one_shot(&phase, plan),
            Err(denial) if matches!(denial.kind(), worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind::Projection(_))));
    }).unwrap();
}
