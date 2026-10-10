//! public query result reconstructs disposable managed view from owner observations.
use super::*;

#[test]
fn public_query_result_reconstructs_disposable_managed_view_from_owner_observations() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
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
            selected
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
    let query = installed_ordered_query(&world);
    let definition = ApplicationDerivedViewDefinition::new(
        "scene-labels",
        OrderedAccountSummaryQuery::reference(),
        ApplicationDerivedViewLimits::bounded(8, 32_768),
    );
    let view = world
        .application
        .open_managed_derived_view(&definition, &query, selected.product())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let result = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    selected
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new()
                                .bind(status_parameter(), "open".to_string())
                                .unwrap(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let retained_product = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let foreign = installed_authorization_world(true);
    foreign
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                world
                    .application
                    .reconstruct_managed_derived_view(
                        &view,
                        foreign
                            .selected_product(&(phase).test_host_request(),)
                            .product(),
                        &result,
                        |row| SceneLabel(row.label().to_string()),
                    )
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::ForeignBranch)
            )
        })
        .expect("the fixture read owns its installed host call");
    let tiny = world
        .application
        .open_managed_derived_view(
            &ApplicationDerivedViewDefinition::new(
                "tiny-scene",
                OrderedAccountSummaryQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 1),
            ),
            &query,
            retained_product.product(),
        )
        .unwrap();
    assert_eq!(
        world
            .application
            .reconstruct_managed_derived_view(&tiny, retained_product.product(), &result, |row| {
                SceneLabel(row.label().to_string())
            })
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
    );
    drop(tiny);
    world
        .application
        .reconstruct_managed_derived_view(&view, retained_product.product(), &result, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    let observed = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    let key = result.observed_sources()[0].managed_derived_view_key();
    assert_eq!(observed.get(&key).unwrap().unwrap().0, "primary");
    view.discard();
    assert!(world
        .application
        .observe_managed_derived_view(&view)
        .is_ok());
    assert_eq!(
        observed.get(&key).err(),
        Some(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
    );
    world
        .application
        .reconstruct_managed_derived_view(&view, retained_product.product(), &result, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    drop(retained_product);
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountLabel::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&"changed".to_string()).unwrap(),
    )]));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::super::super::fixture::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("direct-world-change").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: account.entity_id(),
                        fields,
                    }),
                )),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.observe_managed_derived_view(&view),
        Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
    ));
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
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
            selected
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
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    selected
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new()
                                .bind(status_parameter(), "open".to_string())
                                .unwrap(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let fresh_product = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .reconstruct_managed_derived_view(&view, fresh_product.product(), &fresh, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    let fresh_key = fresh.observed_sources()[0].managed_derived_view_key();
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&fresh_key)
            .unwrap()
            .unwrap()
            .0,
        "changed"
    );
}
