use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountMembershipTag, PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use worth_foundational::facade::AspectFieldLocator;

#[test]
fn certified_membership_changes_retain_clean_arcs_and_read_only_new_or_dirty_entries() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
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
                "reconciled-scene",
                PublicAccountMembershipQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 65_536),
            ),
            &membership_query,
            &entry_query,
            &entry_query,
            selected.product(),
        )
        .unwrap();
    let read_membership = || {
        let current = world.selected_product();
        let principal = current
            .resolve_authenticated_principal(
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = current
            .resolve_entity(
                AccountStatus::reference(),
                "open".to_string(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        world
            .application
            .execute_application_query_one_shot(
                current
                    .admit_application_query(
                        &membership_query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap(),
            )
            .unwrap()
    };
    let read_entry = |key: &str| {
        let current = world.selected_product();
        let principal = current
            .resolve_authenticated_principal(
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = current
            .resolve_entity(
                AccountStatus::reference(),
                key.to_string(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        world
            .application
            .execute_application_query_one_shot(
                current
                    .admit_application_query(
                        &entry_query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap(),
            )
            .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
    };
    let members = |row: &crate::domain_computation::primary_graph::tests::fixture::PublicAccountMembershipResult| {
        row.members.iter().map(|root| (
            *root, row.tag.split('-').next().unwrap().to_string(),
        )).collect::<Vec<_>>()
    };
    let membership = read_membership();
    let keys = world
        .application
        .reconstruct_managed_derived_collection_pair_lazy(
            &view,
            selected.product(),
            &membership,
            members,
            |key| read_entry(key),
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    let key = keys[0].clone();
    let original = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap()
        .get(&key)
        .unwrap()
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let tag_field = AccountMembershipTag::reference();
    let tag_locator = graph
        .layout
        .field_locator(tag_field.entity(), tag_field.aspect(), tag_field.field())
        .unwrap()
        .clone();
    let label_field = AccountLabel::reference();
    let label_locator = graph
        .layout
        .field_locator(
            label_field.entity(),
            label_field.aspect(),
            label_field.field(),
        )
        .unwrap()
        .clone();
    let change =
        |before: &WorthQueryProductBranchLease, locator: &AspectFieldLocator, value: &str| {
            let prepared = graph.managed_derived_views().prepare_publication(
                publication_basis(before),
                &[ViewChange::Aspect(
                    key.root(),
                    locator.aspect().aspect_key().clone(),
                )],
                100_000,
            );
            let fields = AspectFieldPatch::from(BTreeMap::from([(
                locator.clone(),
                StringApplicationValueBinding::encode(&value.to_string()).unwrap(),
            )]));
            super::super::super::fixture::publish_relational_mutation(
                &world,
                WorkerIntentBatch::new("view-reconcile-change").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: key.root(),
                        fields,
                    }),
                )),
            );
            let after = world.selected_product();
            prepared.apply(after.product().selected_commit());
            after
        };
    let after_v2 = change(selected.product(), &tag_locator, "open-v2");
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::MembershipReconciliationRequired)
    );
    let stale = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after_v2.product(),
            &membership,
            members,
            |key| read_entry(key),
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        );
    assert_eq!(
        stale.err(),
        Some(WorthQueryManagedDerivedViewDenial::StaleSource)
    );
    let after_v3 = change(after_v2.product(), &tag_locator, "open-v3");
    let mut first_reads = 0;
    let current = read_membership();
    let retained = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after_v3.product(),
            &current,
            members,
            |token| {
                first_reads += 1;
                read_entry(token)
            },
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    assert_eq!(
        (
            first_reads,
            retained.retained_entries(),
            retained.refreshed_entries(),
            retained.removed_entries()
        ),
        (0, 1, 0, 0)
    );
    assert!(std::sync::Arc::ptr_eq(
        &original,
        &world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .unwrap()
            .unwrap()
    ));

    let after_none = change(after_v3.product(), &tag_locator, "none");
    let current = read_membership();
    assert!(current.rows()[0].members.is_empty());
    let removed = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after_none.product(),
            &current,
            members,
            |key| read_entry(key),
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    assert_eq!((removed.keys().len(), removed.removed_entries()), (0, 1));
    assert!(world
        .application
        .observe_managed_derived_view(&view)
        .unwrap()
        .get(&key)
        .unwrap()
        .is_none());

    let after_open = change(after_none.product(), &tag_locator, "open");
    let current = read_membership();
    let added = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after_open.product(),
            &current,
            members,
            |key| read_entry(key),
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    assert_eq!(
        (added.refreshed_entries(), added.retained_entries()),
        (1, 0)
    );
    assert_eq!(added.keys(), std::slice::from_ref(&key));

    let after_content = change(after_open.product(), &label_locator, "primary-content");
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
    );
    let after_membership = change(after_content.product(), &tag_locator, "open-v4");
    let after_label = change(
        after_membership.product(),
        &label_locator,
        "primary-reconciled",
    );
    let current = read_membership();
    let mut first_reads = 0;
    let refreshed = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after_label.product(),
            &current,
            members,
            |token| {
                first_reads += 1;
                read_entry(token)
            },
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        )
        .unwrap();
    assert_eq!(
        (
            first_reads,
            refreshed.refreshed_entries(),
            refreshed.retained_entries()
        ),
        (1, 1, 0)
    );
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .unwrap()
            .unwrap()
            .0,
        "primary-reconciled"
    );
}
