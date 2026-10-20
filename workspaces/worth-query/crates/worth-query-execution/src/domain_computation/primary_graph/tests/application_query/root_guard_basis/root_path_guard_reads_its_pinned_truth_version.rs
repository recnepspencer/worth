//! root path guard reads its pinned truth version.
use super::*;

#[test]
fn root_path_guard_reads_its_pinned_truth_version() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
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
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(10).unwrap(),
                        Some(NonZeroUsize::new(10_000).unwrap()),
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
                .selected_product(&(phase).test_host_request())
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
    assert_eq!(empty_selection.aspects.len(), 1);
    assert_eq!(empty_selection.entities, [account.entity_id()]);
    let selection = pinned_result.observed_sources()[0]
        .footprint_for_test()
        .root_selection
        .as_ref()
        .expect("pinned path result retains the guard source");
    let guard = selection
        .aspects
        .iter()
        .find(|aspect| aspect.entity == account.entity_id())
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: guard.entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            guard.aspect.clone(),
            CanonicalFieldPath::single(guard.field.clone()),
        ),
        native_revision: guard.native_revision,
    };
    let current = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let unavailable = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: guard.entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            guard.aspect.clone(),
            CanonicalFieldPath::single(guard.field.clone()),
        ),
        native_revision: None,
    };
    let unavailable_currentness = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    unavailable.source_currentness_within(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        current.application_basis().snapshot_handle(),
                        1,
                    )
                })
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        unavailable_currentness,
        Err(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Unavailable)
    ), "an unavailable restored field revision must preserve its typed verification failure");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(
                !graph
                    .integration_handle()
                    .with_runtime(request_execution, |runtime| {
                        fact.source_currentness_within(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            current.application_basis().snapshot_handle(),
                            1,
                        )
                        .unwrap()
                        .0
                        .movement()
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
    let absent_guard = &empty_selection.aspects[0];
    let absent_fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: absent_guard.entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            absent_guard.aspect.clone(),
            CanonicalFieldPath::single(absent_guard.field.clone()),
        ),
        native_revision: absent_guard.native_revision,
    };
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
                "open",
            )
        })
        .expect("the fixture read owns its installed host call");
    let reopened = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(
                !graph
                    .integration_handle()
                    .with_runtime(request_execution, |runtime| {
                        absent_fact
                            .source_currentness_within(
                                runtime,
                                &host_custody.borrowed_request_for_test(),
                                reopened.application_basis().snapshot_handle(),
                                1,
                            )
                            .unwrap()
                            .0
                            .movement()
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
            .idempotency_identity(),
        reopened_result
            .result_set_observation()
            .idempotency_identity()
    );
}
