//! optional owner removal and insertion refresh at the real current product.
use super::*;

#[test]
fn optional_owner_removal_and_insertion_refresh_at_the_real_current_product() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
            let scope = selected
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let query = world
                .application
                .installed_schema()
                .certification_query(OptionalAccountOwnerQuery::reference())
                .unwrap();
            let census = world
                .application
                .installed_schema()
                .certification_query(PublicAccountMembershipQuery::reference())
                .unwrap();
            let view = world
                .application
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "optional-owner",
                        PublicAccountMembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 65_536),
                    ),
                    &census,
                    &query,
                    &query,
                    selected.product(),
                )
                .unwrap();
            let membership = membership::read(
                &host_custody.phase_for_application(&world.application),
                &world,
            );
            let account = membership.rows()[0].members[0];
            let owner = read(
                &host_custody.phase_for_application(&world.application),
                request_execution,
                &world,
                None,
            )
            .ordinary()
            .rows()[0]
                .owner
                .expect("fixture has one native ownership edge");
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
                    |_| {
                        let first = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &query,
                                &access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .map_err(|_| {
                                WorthQueryManagedDerivedViewDenial::QueryExecutionDenied
                            })?;
                        let second = selected
                            .retain_selection(request_execution)
                            .unwrap()
                            .admit_application_query(
                                &host_custody.phase_for_application(&world.application),
                                &query,
                                &access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&request),
                            )
                            .map_err(|_| {
                                WorthQueryManagedDerivedViewDenial::QueryExecutionDenied
                            })?;
                        Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                    },
                    |row| row.account,
                    |row| row.account,
                    |_, row| Owner(row.owner),
                )
                .unwrap();
            let key = &keys[0];
            assert_eq!(
                world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(key)
                    .unwrap()
                    .unwrap()
                    .0,
                Some(owner)
            );
            let graph = world.application.runtime.primary_graph().unwrap();
            let kind = graph
                .layout
                .relation(AccountOwner::reference().name())
                .unwrap()
                .kind;
            let edge = graph
                .integration_handle()
                .with_runtime_mut(request_execution, |runtime| {
                    runtime
                        .read_truth()
                        .visible_relations_of_kind(
                            request_execution,
                            kind,
                            selected.application_basis().snapshot_handle().version_id(),
                        )
                        .into_iter()
                        .find(|edge| edge.target == account)
                        .unwrap()
                        .relation_id
                });
            let removed = graph.managed_derived_views().prepare_publication(
                publication_basis(selected.product()),
                &[ViewChange::Adjacency(account, kind, 1)],
                100_000,
            );
            drop(selected);
            super::super::super::super::fixture::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("remove-optional-owner").push(MutationIntent::Relation(
                    RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: edge }),
                )),
            );
            let absent = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            removed.apply(absent.product().selected_commit());
            assert_eq!(
                world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(key)
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
            );
            assert_eq!(
                refresh(
                    &host_custody.phase_for_application(&world.application),
                    request_execution,
                    &world,
                    &view,
                    key
                )
                .0,
                None
            );
            let inserted = graph.managed_derived_views().prepare_publication(
                publication_basis(absent.product()),
                &[ViewChange::Adjacency(account, kind, 1)],
                100_000,
            );
            drop(absent);
            super::super::super::super::fixture::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("insert-optional-owner").push(MutationIntent::Create(
                    CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: kind,
                        client_key: ClientKey::raw("restored-owner"),
                        source: EntityReference::Existing(owner),
                        target: EntityReference::Existing(account),
                        fields: AspectFieldPatch::default(),
                    }),
                )),
            );
            let present = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            inserted.apply(present.product().selected_commit());
            assert_eq!(
                world
                    .application
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(key)
                    .err(),
                Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
            );
            assert_eq!(
                refresh(
                    &host_custody.phase_for_application(&world.application),
                    request_execution,
                    &world,
                    &view,
                    key
                )
                .0,
                Some(owner)
            );
        })
        .expect("the fixture block owns its installed host call")
}
