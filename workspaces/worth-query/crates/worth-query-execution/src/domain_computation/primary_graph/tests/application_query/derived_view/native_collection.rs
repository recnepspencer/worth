use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountSummaryResult, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::WorthQueryManagedDerivedCollectionBatchRefreshDenial;

mod batch_refresh;

#[test]
fn one_collection_cold_reads_two_declared_queries_then_refreshes_only_dirty_entry() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let open = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let unrelated = selected
        .resolve_entity(
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
    let membership = world
        .application
        .execute_application_query_one_shot(
            world
                .selected_product()
                .admit_application_query(
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
        .reconstruct_managed_derived_collection_pair_lazy(
            &view,
            selected.product(),
            &membership,
            |row| {
                row.members
                    .iter()
                    .map(|root| (*root, row.tag.clone()))
                    .collect()
            },
            |occurrence_key| {
                let occurrence_scope = selected
                    .resolve_entity(
                        AccountStatus::reference(),
                        occurrence_key.clone(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let occurrence_access =
                    WorthQueryApplicationQueryAccessContext::new(&principal, &occurrence_scope);
                world
                    .application
                    .execute_application_query_one_shot(
                        world
                            .selected_product()
                            .admit_application_query(
                                &entry_query,
                                &occurrence_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap(),
                    )
                    .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
            },
            |row| {
                let body_scope = selected
                    .resolve_entity(
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
                        world
                            .selected_product()
                            .admit_application_query(
                                &entry_query,
                                &body_access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap(),
                    )
                    .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
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
        .reconstruct_managed_derived_collection_pair_lazy(
            &view,
            selected.product(),
            &membership,
            |row| {
                row.members
                    .iter()
                    .map(|root| (*root, row.tag.clone()))
                    .collect()
            },
            batch_refresh::deny_first,
            batch_refresh::deny_second,
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
        super::super::super::fixture::publish_relational_mutation(
            &world,
            WorkerIntentBatch::new("native-view-label").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
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
    let after = world.selected_product();
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
    let fresh = world.selected_product();
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
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let open = fresh
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
    let admit_entry = || {
        world
            .selected_product()
            .admit_application_query(
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
                AccountStatus::reference(),
                row.status().to_string(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let body_access = WorthQueryApplicationQueryAccessContext::new(&principal, &body_scope);
        world
            .application
            .execute_application_query_one_shot(
                world
                    .selected_product()
                    .admit_application_query(
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
    let first_work = world
        .application
        .execute_application_query_one_shot(admit_entry())
        .unwrap()
        .receipt()
        .work()
        .total_work_units();
    let first_memory = std::cell::Cell::new(0);
    let projections = std::cell::Cell::new(0);
    let snapshot = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    let (application, fresh_product, _) = fresh.into_parts();
    batch_refresh::refuse_then_refresh(
        first_work,
        &first_memory,
        &projections,
        &snapshot,
        &key,
        |batch, second_batch| {
            world
                .application
                .refresh_managed_derived_collection_pair_entry_in_batch(
                    &view,
                    &fresh_product,
                    &key,
                    admit_entry(),
                    batch,
                    |row: &AccountSummaryResult, batch| {
                        first_memory.set(batch.observe().retained_bytes());
                        let selected = application
                            .on_product(fresh_product.retained_clone())
                            .unwrap();
                        let scope = selected
                            .resolve_entity(
                                AccountStatus::reference(),
                                row.status().to_owned(),
                                &request,
                                WorthQueryPrincipalResolutionMode::Ordinary,
                            )
                            .unwrap();
                        let access =
                            WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
                        let plan = selected
                            .admit_application_query(
                                &entry_query,
                                &access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .unwrap();
                        world
                            .application
                            .execute_application_query_one_shot_in_batch(plan, second_batch)
                            .map_err(WorthQueryManagedDerivedCollectionBatchRefreshDenial::Read)
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
}
