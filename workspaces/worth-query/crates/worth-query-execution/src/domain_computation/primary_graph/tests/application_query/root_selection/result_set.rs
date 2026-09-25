use super::*;

#[test]
fn empty_path_union_stales_when_a_matching_edge_is_inserted() {
    let world = installed_authorization_world(true);
    let request = super::super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let primary_kind = graph
        .layout
        .relation(super::super::super::fixture::AccountPrimaryActivity::reference().name())
        .unwrap()
        .kind;
    let selected = world.selected_product();
    let (relations, primary_target) = graph.integration_handle().with_runtime(|runtime| {
        let primary_target = runtime
            .read_truth()
            .visible_relations_of_kind(
                primary_kind,
                selected.application_basis().snapshot_handle().version_id(),
            )
            .into_iter()
            .find(|relation| relation.source == account.entity_id())
            .unwrap()
            .target;
        let relations = [
            super::super::super::fixture::AccountPrimaryActivity::reference().name(),
            super::super::super::fixture::AccountSecondaryActivity::reference().name(),
            super::super::super::fixture::AccountAllActivity::reference().name(),
        ]
        .into_iter()
        .flat_map(|name| {
            let kind = graph.layout.relation(name).unwrap().kind;
            runtime
                .read_truth()
                .visible_relations_of_kind(
                    kind,
                    selected.application_basis().snapshot_handle().version_id(),
                )
                .into_iter()
                .filter(|relation| relation.source == account.entity_id())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
        (relations, primary_target)
    });
    assert_eq!(relations.len(), 4);
    drop(selected);
    let mut deletion = WorkerIntentBatch::new("empty-path-set");
    for relation in relations {
        deletion = deletion.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: relation.relation_id,
            },
        )));
    }
    super::super::super::fixture::publish_relational_mutation(&world, deletion);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let empty = world
        .application
        .execute_application_query_one_shot(
            world
                .selected_product()
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    assert!(empty.rows().is_empty());
    assert!(empty.observed_sources().is_empty());
    let source = empty.result_set_observation().selection_for_test();
    assert_eq!(source.adjacencies.len(), 3);
    let observed = source
        .adjacencies
        .iter()
        .find(|adjacency| adjacency.relation_kind == primary_kind)
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: observed.relation_kind,
        anchor: observed.anchor,
        direction: observed.direction,
        native_revision: observed.native_revision,
        comparison_work_limit: observed.comparison_work_limit,
        endpoints: Vec::new(),
    };
    super::super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("insert-matching-path").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: primary_kind,
                client_key: ClientKey::raw("new-primary-path"),
                source: EntityReference::Existing(account.entity_id()),
                target: EntityReference::Existing(primary_target),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
    let current = world.selected_product();
    assert!(!graph.integration_handle().with_runtime(|runtime| {
        fact.source_currentness_in(runtime, current.application_basis().snapshot_handle(), 1)
            .unwrap()
            .0
    }));
    let matching = world
        .application
        .execute_application_query_one_shot(
            current
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(matching.rows().len(), 1);
    assert_ne!(
        empty.result_set_observation().idempotency_identity(),
        matching.result_set_observation().idempotency_identity()
    );
}
