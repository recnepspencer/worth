use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountMembershipTag, PublicAccountMembershipQuery, PublicAccountMembershipResult,
    PublicScopedAccountSummaryQuery,
};

#[test]
fn changed_member_token_for_same_entity_refreshes_entry_and_duplicate_members_fail_closed() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let execution = serial_request(&request);
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
                "reconciled-token",
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
    let read_entry = |token: &str| {
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
                token.split('-').next().unwrap().to_string(),
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
    let members = |row: &PublicAccountMembershipResult| {
        row.members
            .iter()
            .map(|root| (*root, row.tag.clone()))
            .collect::<Vec<_>>()
    };
    let membership = read_membership();
    let pair_principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let pair_scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let pair_access = WorthQueryApplicationQueryAccessContext::new(&pair_principal, &pair_scope);
    let keys = world
        .application
        .reconstruct_managed_derived_collection_pair(
            worth_execution::ExecutionRequest::serial(&execution),
            &view,
            selected.product(),
            &membership,
            members,
            |_: &String| {
                let first = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
                        &entry_query,
                        &pair_access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap();
                let second = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
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
    let key = keys[0].clone();
    let old = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap()
        .get(&key)
        .unwrap()
        .unwrap();
    let denied = world
        .application
        .reconstruct_managed_derived_collection_pair(
            worth_execution::ExecutionRequest::serial(&execution),
            &view,
            selected.product(),
            &membership,
            members,
            |_: &String| {
                let first = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
                        &entry_query,
                        &pair_access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap();
                let second = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
                        &entry_query,
                        &pair_access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap();
                let pair = WorthQueryDerivedPairReadPlans::new(first, second);
                Err::<(), _>(WorthQueryManagedDerivedViewDenial::QueryExecutionDenied).and(Ok(pair))
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
        &old,
        &world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .unwrap()
            .unwrap()
    ));
    let graph = world.application.runtime.primary_graph().unwrap();
    let tag_field = AccountMembershipTag::reference();
    let tag_locator = graph
        .layout
        .field_locator(tag_field.entity(), tag_field.aspect(), tag_field.field())
        .unwrap()
        .clone();
    let prepared = graph.managed_derived_views().prepare_publication(
        publication_basis(selected.product()),
        &[ViewChange::Aspect(
            key.root(),
            tag_locator.aspect().aspect_key().clone(),
        )],
        100_000,
    );
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        tag_locator,
        StringApplicationValueBinding::encode(&"open-v2".to_string()).unwrap(),
    )]));
    super::super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("member-token-change").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: key.root(),
                fields,
            }),
        )),
    );
    let after = world.selected_product();
    prepared.apply(after.product().selected_commit());
    let current = read_membership();
    let mut denied_reads = 0;
    let duplicate = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after.product(),
            &current,
            |row: &PublicAccountMembershipResult| {
                row.members
                    .iter()
                    .flat_map(|root| [(*root, row.tag.clone()), (*root, row.tag.clone())])
                    .collect()
            },
            |token| {
                denied_reads += 1;
                read_entry(token)
            },
            |row| read_entry(row.status()),
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| SceneLabel(body.label().to_string()),
        );
    assert_eq!(
        duplicate.err(),
        Some(WorthQueryManagedDerivedViewDenial::IncompleteDependencies)
    );
    assert_eq!(denied_reads, 0);
    let mut token_reads = 0;
    let changed = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after.product(),
            &current,
            members,
            |token| {
                token_reads += 1;
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
            token_reads,
            changed.refreshed_entries(),
            changed.retained_entries()
        ),
        (1, 1, 0)
    );
    assert!(!std::sync::Arc::ptr_eq(
        &old,
        &world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&key)
            .unwrap()
            .unwrap()
    ));
}
