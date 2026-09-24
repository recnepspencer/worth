use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountSummaryResult, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};

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
    assert_eq!(membership.rows()[0].tag, "member");
    assert_eq!(membership.rows()[0].members, vec![open.entity_id()]);
    let keys = world
        .application
        .reconstruct_managed_derived_collection_pair(
            &view,
            selected.product(),
            &membership,
            vec![world
                .selected_product()
                .admit_application_query(
                    &entry_query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap()],
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
            |row| row.members.clone(),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    assert_eq!(keys.len(), 1);
    let key = keys[0].clone();
    let original = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap()
        .get(&key)
        .unwrap()
        .unwrap();
    assert_eq!(original.0, "primary");

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
    let refreshed = world
        .application
        .refresh_managed_derived_collection_pair_entry(
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
}
