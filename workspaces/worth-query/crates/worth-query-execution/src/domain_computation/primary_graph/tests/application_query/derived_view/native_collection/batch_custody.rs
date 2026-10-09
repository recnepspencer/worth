//! Closed pair custody and late batch-refresh refusals preserve the retained entry.
use super::*;

#[test]
fn shared_pair_claims_survive_projection_and_late_refusal_is_atomic() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let execution = serial_request(&request);
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
    let ordinary = world
        .application
        .execute_application_query_one_shot(
            selected
                .retain_selection()
                .unwrap()
                .admit_application_query(
                    &entry_query,
                    &pair_access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    let read_work = ordinary.receipt().work().total_work_units();
    drop(ordinary);
    let nz = |value| std::num::NonZeroUsize::new(value).unwrap();
    let closed_batch =
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryBatchAdmission::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryBatchLimits::new(
                nz(2),
                nz(100_000),
                nz(262_144),
                nz(4_096),
            ),
        );
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
                Ok(WorthQueryDerivedPairReadPlans::in_batch(
                    first,
                    second,
                    &closed_batch,
                ))
            },
            |row| row.status().to_string(),
            |row| row.status().to_string(),
            |_, body| {
                assert!(
                    closed_batch.observe().retained_bytes() > 0,
                    "closed read claims survive owner projection"
                );
                SceneLabel(body.label().to_string())
            },
        )
        .unwrap();
    assert_eq!(keys.len(), 1);
    let key = keys[0].clone();
    assert_eq!(closed_batch.observe().retained_bytes(), 0);
    assert_eq!(
        closed_batch.observe().read_work_units(),
        read_work.checked_mul(2).unwrap(),
        "both closed reads examine the same query and root as the ordinary receipt",
    );
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
            &world,
            WorkerIntentBatch::new("native-view-label").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
            )),
        );
    };
    let prepared = graph.managed_derived_views().prepare_publication(
        publication_basis(selected.product()),
        &[ViewChange::Aspect(
            open.entity_id(),
            locator.aspect().aspect_key().clone(),
        )],
        100_000,
    );
    drop(selected);
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
    let first = world
        .application
        .execute_application_query_one_shot(admit_entry())
        .unwrap();
    let first_work = first.receipt().work().total_work_units();
    drop(first);
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
        |first_batch, batch, second_batch| {
            let first = world
                .application
                .execute_application_query_one_shot_in_batch(admit_entry(), first_batch)
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
