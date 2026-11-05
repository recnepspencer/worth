//! root path guard reads its pinned truth version.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use crate::domain_computation::primary_graph::tests::application_query::root_guard_basis::*;

#[test]
fn root_path_guard_reads_its_pinned_truth_version() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
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
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let pinned = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id_for_test(),
                "closed",
            )
        })
        .expect("the fixture read owns its installed host call");
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let pinned_plan = world
        .application
        .with_host_advancement(|phase| {
            pinned
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(10).unwrap(),
                        &request,
                    ),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let pinned_result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, pinned_plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let current_plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let current_result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, current_plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(pinned_result.rows().len(), 2);
    assert!(current_result.rows().is_empty());
    let empty_selection = current_result.result_set_observation().selection_for_test();
    assert_eq!((*empty_selection.aspects_for_test()).len(), 1);
    assert_eq!(
        (*empty_selection.entities_for_test()),
        [account.entity_id_for_test()]
    );
    let selection = pinned_result.observed_sources()[0]
        .footprint_for_test()
        .root_selection
        .as_ref()
        .expect("pinned path result retains the guard source");
    let guard = (*selection.aspects_for_test())
        .iter()
        .find(|aspect| (*aspect.entity_for_test()) == account.entity_id_for_test())
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: (*guard.entity_for_test()),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            (*guard.aspect_for_test()).clone(),
            CanonicalFieldPath::single((*guard.field_for_test()).clone()),
        ),
        native_revision: guard.native_revision,
    };
    let current = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let unavailable = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: (*guard.entity_for_test()),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            (*guard.aspect_for_test()).clone(),
            CanonicalFieldPath::single((*guard.field_for_test()).clone()),
        ),
        native_revision: None,
    };
    let unavailable_currentness = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            graph
                .integration_handle_for_test()
                .with_runtime(request_execution, |runtime| {
                    unavailable.source_currentness_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        current.application_basis_for_test().snapshot_handle(),
                    )
                })
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        unavailable_currentness.unwrap(),
        Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure::Unavailable)
    ), "an unavailable restored field revision must preserve its typed verification failure");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                !graph
                    .integration_handle_for_test()
                    .with_runtime(request_execution, |runtime| {
                        fact.source_currentness_in_for_test(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            current.application_basis_for_test().snapshot_handle(),
                        )
                        .unwrap()
                        .unwrap()
                        .movement_for_test()
                            == Movement::Unmoved
                    }),
                "changed root-path guard must stale the selected source"
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        pinned_result.receipt().basis_posture(),
        WorthQueryApplicationQueryBasisPosture::SelectedProduct
    );
    let absent_guard = &(*empty_selection.aspects_for_test())[0];
    let absent_fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: (*absent_guard.entity_for_test()),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            (*absent_guard.aspect_for_test()).clone(),
            CanonicalFieldPath::single((*absent_guard.field_for_test()).clone()),
        ),
        native_revision: absent_guard.native_revision,
    };
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id_for_test(),
                "open",
            )
        })
        .expect("the fixture read owns its installed host call");
    let reopened = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                !graph
                    .integration_handle_for_test()
                    .with_runtime(request_execution, |runtime| {
                        absent_fact
                            .source_currentness_in_for_test(
                                runtime,
                                &host_custody.borrowed_request_for_test(),
                                reopened.application_basis_for_test().snapshot_handle(),
                            )
                            .unwrap()
                            .unwrap()
                            .movement_for_test()
                            == Movement::Unmoved
                    }),
                "a matching guard edit must stale an empty result-set observation"
            )
        })
        .expect("the fixture read owns its installed host call");
    let reopened_result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    reopened
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(reopened_result.rows().len(), 2);
    assert_ne!(
        current_result
            .result_set_observation()
            .idempotency_identity_for_test(),
        reopened_result
            .result_set_observation()
            .idempotency_identity_for_test()
    );
}
