//! Closed pair custody and late batch-refresh refusals preserve the retained entry.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::native_collection::*;

#[test]
fn shared_pair_claims_survive_projection_and_late_refusal_is_atomic() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let request_execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            );
            let principal = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let open = selected
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    AccountStatus::reference(),
                    "open".to_string(),
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
                    &host_custody.phase_for_application_for_test(&world.application),
                    world
                        .selected_product(
                            &(host_custody.phase_for_application_for_test(&world.application))
                                .test_host_request_for_test(),
                        )
                        .admit_application_query(
                            &host_custody.phase_for_application_for_test(&world.application),
                            &membership_query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(membership_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                        )
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(membership.rows().len(), 1);
            assert_eq!(membership.rows()[0].tag, "open");
            assert_eq!(membership.rows()[0].members, vec![open.entity_id_for_test()]);
            let ordinary = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application_for_test(&world.application),
                    selected
                        .retain_selection_for_test(request_execution)
                        .unwrap()
                        .admit_application_query(
                            &host_custody.phase_for_application_for_test(&world.application),
                            &entry_query,
                            &pair_access,
                            ApplicationQueryParameterSet::new(),
                            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                        )
                        .unwrap(),
                )
                .unwrap();
            drop(ordinary);

            let closed_batch = world
                .application
                .application_query_batch_from_installed_reads(&[world
                    .application
                    .plan_installed_query_batch_reads(
                        &entry_query,
                        2,
                        std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"),
                    )
                    .unwrap()])
                .unwrap();
            let keys = world
                .application
                .reconstruct_managed_derived_collection_pair(
                    *request_execution,
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
                            .retain_selection_for_test(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application_for_test(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                            )
                            .unwrap();
                        let second = selected
                            .retain_selection_for_test(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application_for_test(&world.application),
                                &entry_query,
                                &pair_access,
                                ApplicationQueryParameterSet::new(),
                                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                            )
                            .unwrap();
                        Ok(WorthQueryDerivedPairReadPlans::in_batch(
                            first,
                            second,
                            &closed_batch,
                        ))
                    },
                    |row| row.status().to_string(),
                    |row| row.status().to_string(),
                    |_, body| SceneLabel(body.label().to_string()),
                )
                .unwrap();

            assert_eq!(keys.len(), 1);
            let key = keys[0].clone();

            let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
            let field = AccountLabel::reference();
            let locator = (*graph
                .layout_for_test())
                .field_locator_for_test(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            let change_label = |entity_id, label: &str| {
                let fields = AspectFieldPatch::from(BTreeMap::from([(
                    locator.clone(),
                    StringApplicationValueBinding::encode(&label.to_string()).unwrap(),
                )]));
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
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
            let prepared = graph.managed_derived_views_for_test().prepare_publication_for_test(
                publication_basis(selected.product()),
                &[ViewChange::Aspect(
                    open.entity_id_for_test(),
                    locator.aspect().aspect_key().clone(),
                )],
                100_000,
            );
            drop(selected);
            change_label(open.entity_id_for_test(), "primary-changed");
            let fresh = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
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
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let open = fresh
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
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
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    )
                    .admit_application_query(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &entry_query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                    )
                    .unwrap()
            };
            let read_second = |row: &AccountSummaryResult| {
                let body_scope = fresh
                    .resolve_entity(
                        &host_custody.phase_for_application_for_test(&world.application),
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
                        &host_custody.phase_for_application_for_test(&world.application),
                        world
                            .selected_product(
                                &(host_custody.phase_for_application_for_test(&world.application))
                                    .test_host_request_for_test(),
                            )
                            .admit_application_query(
                                &host_custody.phase_for_application_for_test(&world.application),
                                &entry_query,
                                &body_access,
                                ApplicationQueryParameterSet::new(),
                                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
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
            let first = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application_for_test(&world.application),
                    admit_entry(),
                )
                .unwrap();
            let first_work = first.receipt().work().total_work_units();
            drop(first);

            let projections = std::cell::Cell::new(0);
            let snapshot = world
                .application
                .observe_managed_derived_view(&view)
                .unwrap();
            let (application, fresh_product, _) = fresh.into_parts_for_test();
            crate::domain_computation::primary_graph::tests::application_query::derived_view::native_collection::batch_refresh::refuse_then_refresh(
                || {
                    world
                        .application
                        .application_query_batch_from_installed_reads(&[world
                            .application
                            .plan_installed_query_batch_reads(
                                &entry_query,
                                2,
                                std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"),
                            )
                            .unwrap()])
                        .unwrap()
                },
                &projections,
                &snapshot,
                &key,
                |first_batch, batch, second_batch| {
                    let first = world
                        .application
                        .execute_application_query_one_shot_in_batch(
                            &host_custody.phase_for_application_for_test(&world.application),
                            admit_entry(),
                            first_batch,
                        )
                        .map_err(WorthQueryManagedDerivedCollectionBatchRefreshDenial::Read)?;
                    assert_eq!(
                        first.result().receipt().work().total_work_units(),
                        first_work
                    );
                    world
                        .application
                        .refresh_managed_derived_collection_pair_entry_from_batch_result(
                            &view,
                            &fresh_product,
                            &key,
                            first,
                            batch,
                            |row: &AccountSummaryResult, _| {
                                let selected = application
                                    .on_product_for_test(request_execution, fresh_product.retained_clone_for_test())
                                    .unwrap();
                                let scope = selected
                                    .resolve_entity(
                                        &host_custody.phase_for_application_for_test(&world.application),
                                        AccountStatus::reference(),
                                        row.status().to_owned(),
                                        &request,
                                        WorthQueryPrincipalResolutionMode::Ordinary,
                                    )
                                    .unwrap();
                                let access = WorthQueryApplicationQueryAccessContext::new(
                                    &principal, &scope,
                                );
                                let plan = selected
                                    .admit_application_query(
                                        &host_custody.phase_for_application_for_test(&world.application),
                                        &entry_query,
                                        &access,
                                        ApplicationQueryParameterSet::new(),
                                        worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(entry_query.read_graph().maximum_result_count()).expect("the fixture uses a nonzero installed query result contract"), &request),
                                    )
                                    .unwrap();
                                world
                                    .application
                                    .execute_application_query_one_shot_in_batch(
                                        &host_custody.phase_for_application_for_test(&world.application),
                                        plan,
                                        second_batch,
                                    )
                                    .map_err(
                                        WorthQueryManagedDerivedCollectionBatchRefreshDenial::Read,
                                    )
                            },
                            |row| row.status().to_string(),
                            |row| row.status().to_string(),
                            |_, body| {
                                projections.set(projections.get() + 1);
                                SceneLabel(body.label().to_string())
                            },
                        )
                },
            );
        })
        .expect("the fixture block owns its installed host call")
}
