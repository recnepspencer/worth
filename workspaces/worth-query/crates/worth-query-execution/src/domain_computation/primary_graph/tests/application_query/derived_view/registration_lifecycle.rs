use super::*;

#[test]
fn governed_query_cannot_open_a_persistent_view_without_read_reauthorization() {
    let world = installed_authorization_world(true);
    let query = installed_query(&world);
    let definition = ApplicationDerivedViewDefinition::new(
        "governed-account",
        AccountSummaryQuery::reference(),
        ApplicationDerivedViewLimits::bounded(8, 4096),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let denial = world
                .application
                .open_managed_derived_view::<_, _, _, _, SceneLabel>(
                    &definition,
                    &query,
                    selected.product(),
                )
                .err();
            assert_eq!(
                denial,
                Some(WorthQueryManagedDerivedViewDenial::AuthorizationRequired)
            );
        })
        .expect("the fixture block owns its installed host call")
}

mod public_query_result_reconstructs_disposable_managed_view_from_ow;

#[test]
fn prepared_view_publication_preserves_unaffected_arc_and_colds_membership_change() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let principal = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let open = selected
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let unrelated = selected
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "unrelated".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = installed_ordered_query(&world);
            let view = world
                .application
                .open_managed_derived_view(
                    &ApplicationDerivedViewDefinition::new(
                        "selective-scene",
                        OrderedAccountSummaryQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 16_384),
                    ),
                    &query,
                    selected.product(),
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
            let result = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application(&world.application),
                    selected
                        .admit_application_query(
                            &host_custody.phase_for_application(&world.application),
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new()
                                .bind(status_parameter(), "open".to_string())
                                .unwrap(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap();
            let key = result.observed_sources()[0].managed_derived_view_key();
            let retained_product = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            assert_eq!(open.entity_id(), key.root());
            world
                .application
                .reconstruct_managed_derived_view(
                    &view,
                    retained_product.product(),
                    &result,
                    |row| SceneLabel(row.label().to_string()),
                )
                .unwrap();
            let original = world
                .application
                .observe_managed_derived_view(&view)
                .unwrap()
                .get(&key)
                .unwrap()
                .unwrap();
            let graph = world.application.runtime.primary_graph().unwrap();
            let field = AccountLabel::reference();
            let locator = graph
                .layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            let change_label = |entity_id, label: &str| {
                let fields = AspectFieldPatch::from(BTreeMap::from([(
                    locator.clone(),
                    StringApplicationValueBinding::encode(&label.to_string()).unwrap(),
                )]));
                super::super::super::fixture::publish_relational_mutation(
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    WorkerIntentBatch::new("view-selective-change").push(MutationIntent::Entity(
                        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                            entity_id,
                            fields,
                        }),
                    )),
                );
            };
            let prepared = graph.managed_derived_views().prepare_publication(
                publication_basis(retained_product.product()),
                &[ViewChange::Aspect(
                    unrelated.entity_id(),
                    locator.aspect().aspect_key().clone(),
                )],
                100_000,
            );
            change_label(unrelated.entity_id(), "other-changed");
            let after = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            prepared.apply(after.product().selected_commit());
            let retained = world
                .application
                .observe_managed_derived_view(&view)
                .unwrap()
                .get(&key)
                .unwrap()
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&original, &retained));

            let prepared = graph.managed_derived_views().prepare_publication(
                publication_basis(after.product()),
                &[ViewChange::Aspect(
                    open.entity_id(),
                    locator.aspect().aspect_key().clone(),
                )],
                100_000,
            );
            change_label(open.entity_id(), "primary-changed");
            let after = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            prepared.apply(after.product().selected_commit());
            assert_eq!(
                world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&key)
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
            );
        })
        .expect("the fixture block owns its installed host call")
}
