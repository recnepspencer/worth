use super::*;

#[test]
fn one_collection_cold_reads_two_declared_queries_then_refreshes_only_dirty_entry() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let execution = serial_request(&request);
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let request_execution = &borrowed_execution;
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
            let membership_query = world
                .application
                .installed_schema()
                .certification_query(PublicAccountMembershipQuery::reference())
                .unwrap();
            let entry_query = world
                .application
                .installed_schema()
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let view = world
                .application
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "native-scene",
                        PublicAccountMembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 32_768),
                    ),
                    &membership_query,
                    &entry_query,
                    &entry_query,
                    selected.product(),
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
            let pair_access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
            let membership = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application(&world.application),
                    world
                        .selected_product(
                            &(host_custody.phase_for_application(&world.application))
                                .test_host_request(),
                        )
                        .admit_application_query(
                            &host_custody.phase_for_application(&world.application),
                            &membership_query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(membership.rows().len(), 1);
            assert_eq!(membership.rows()[0].tag, "open");
            assert_eq!(membership.rows()[0].members, vec![open.entity_id()]);
            let keys = world
                .application
                .reconstruct_managed_derived_collection_pair(
                    worth_execution::ExecutionRequest::serial(&execution),
                    &view,
                    selected.product(),
                    &membership,
                    |row| {
                        row.members
                            .iter()
                            .map(|root| (*root, row.tag.clone()))
                            .collect()
                    },
                    |_: &String| {
                        let first = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap();
                        let second = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap();
                        Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                    },
                    |row| row.status().to_string(),
                    |row| row.status().to_string(),
                    |_, body| SceneLabel(body.label().to_string()),
                )
                .unwrap();
            assert_eq!(keys.len(), 1);
            let key = keys[0].clone();
            let original_snapshot = world
                .application
                .observe_managed_derived_view(&view)
                .unwrap();
            assert_eq!(
                original_snapshot.selected_commit(),
                selected.product().selected_commit()
            );
            let original = original_snapshot.get(&key).unwrap().unwrap();
            assert_eq!(original.0, "primary");
            let denied = world
                .application
                .reconstruct_managed_derived_collection_pair(
                    worth_execution::ExecutionRequest::serial(&execution),
                    &view,
                    selected.product(),
                    &membership,
                    |row| {
                        row.members
                            .iter()
                            .map(|root| (*root, row.tag.clone()))
                            .collect()
                    },
                    |_: &String| {
                        let first = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap();
                        let second = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap();
                        let pair = WorthQueryDerivedPairReadPlans::new(first, second);
                        Err::<(), _>(WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
                            .and(Ok(pair))
                    },
                    |row| row.status().to_string(),
                    |row| row.status().to_string(),
                    |_, body| SceneLabel(body.label().to_string()),
                );
            assert_eq!(
                denied.err(),
                Some(WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
            );
            assert!(std::sync::Arc::ptr_eq(
                &original,
                &world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&key)
                    .unwrap()
                    .unwrap(),
            ));

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
                super::super::super::super::fixture::publish_relational_mutation(
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    WorkerIntentBatch::new("native-view-label").push(MutationIntent::Entity(
                        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                            entity_id,
                            fields,
                        }),
                    )),
                );
            };
            let prepared = graph.managed_derived_views().prepare_publication(
                publication_basis(selected.product()),
                &[ViewChange::Aspect(
                    unrelated.entity_id(),
                    locator.aspect().aspect_key().clone(),
                )],
                100_000,
            );
            drop(selected);
            change_label(unrelated.entity_id(), "other-changed");
            let after = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            prepared.apply(after.product().selected_commit());
            assert_ne!(
                original_snapshot.selected_commit(),
                after.product().selected_commit()
            );
            assert_eq!(
                original_snapshot.get(&key).err(),
                Some(WorthQueryManagedDerivedViewDenial::StaleSource)
            );
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
            drop(after);
            change_label(open.entity_id(), "primary-changed");
            let fresh = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            prepared.apply(fresh.product().selected_commit());
            assert_eq!(
                world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&key)
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired),
            );
            let principal = fresh
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let open = fresh
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
            let admit_entry = || {
                world
                    .selected_product(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                    )
                    .admit_application_query(
                        &host_custody.phase_for_application(&world.application),
                        &entry_query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap()
            };
            let read_second = |row: &AccountSummaryResult| {
                let body_scope = fresh
                    .resolve_entity(
                        &host_custody.phase_for_application(&world.application),
                        AccountStatus::reference(),
                        row.status().to_string(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let body_access =
                    WorthQueryApplicationQueryAccessContext::new(&principal, &body_scope);
                world
                    .application
                    .execute_application_query_one_shot(
                        &host_custody.phase_for_application(&world.application),
                        world
                            .selected_product(
                                &(host_custody.phase_for_application(&world.application))
                                    .test_host_request(),
                            )
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &entry_query,
                                &body_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap(),
                    )
                    .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
            };
            assert_eq!(
                world
                    .application
                    .refresh_managed_derived_collection_pair_entry(
                        request_execution,
                        &view,
                        fresh.product(),
                        &key,
                        admit_entry(),
                        read_second,
                        |_| "wrong-link".to_string(),
                        |row| row.status().to_string(),
                        |_, body| SceneLabel(body.label().to_string()),
                    )
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::IncompleteDependencies),
            );
            let refreshed = world
                .application
                .refresh_managed_derived_collection_pair_entry(
                    request_execution,
                    &view,
                    fresh.product(),
                    &key,
                    admit_entry(),
                    read_second,
                    |row| row.status().to_string(),
                    |row| row.status().to_string(),
                    |_, body| SceneLabel(body.label().to_string()),
                )
                .unwrap();
            assert_eq!(refreshed.0, "primary-changed");
            assert!(std::sync::Arc::ptr_eq(
                &refreshed,
                &world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&key)
                    .unwrap()
                    .unwrap(),
            ));
        })
        .expect("the fixture block owns its installed host call")
}
