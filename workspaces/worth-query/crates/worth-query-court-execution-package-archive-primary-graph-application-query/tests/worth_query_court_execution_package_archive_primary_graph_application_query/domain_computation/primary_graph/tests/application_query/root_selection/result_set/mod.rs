use crate::domain_computation::primary_graph::tests::application_query::root_selection::*;

#[test]
fn empty_path_union_stales_when_a_matching_edge_is_inserted() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let primary_kind = graph
        .layout
        .relation(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity::reference().name())
        .unwrap()
        .kind;
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    let (relations, primary_target) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    let primary_target = runtime
                        .read_truth()
                        .visible_relations_of_kind(
                            &host_custody.borrowed_request_for_test(),
                            primary_kind,
                            selected.application_basis_for_test().snapshot_handle().version_id(),
                        )
                        .expect("fixture read request remains live")
                        .into_iter()
                        .find(|relation| relation.source == account.entity_id_for_test())
                        .unwrap()
                        .target;
                    let relations = [
                        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity::reference().name(),
                        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountSecondaryActivity::reference().name(),
                        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountAllActivity::reference().name(),
                    ]
                    .into_iter()
                    .flat_map(|name| {
                        let kind = graph.layout.relation(name).unwrap().kind;
                        runtime
                            .read_truth()
                            .visible_relations_of_kind(
                                &host_custody.borrowed_request_for_test(),
                                kind,
                                selected.application_basis_for_test().snapshot_handle().version_id(),
                            )
                            .expect("fixture read request remains live")
                            .into_iter()
                            .filter(|relation| relation.source == account.entity_id_for_test())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                    (relations, primary_target)
                })
        })
        .expect("the fixture read owns its installed host call");
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
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                deletion,
            )
        })
        .expect("the fixture read owns its installed host call");
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let empty = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    world
                        .selected_product(&(phase).test_host_request_for_test())
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(empty.rows().is_empty());
    assert!(empty.observed_sources().is_empty());
    let source = empty.result_set_observation().selection_for_test();
    assert_eq!(source.adjacencies.len(), 3);
    let observed = source
        .adjacencies
        .iter()
        .find(|adjacency| (*adjacency.relation_kind_for_test()) == primary_kind)
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: (*observed.relation_kind_for_test()),
        anchor: (*observed.anchor_for_test()),
        direction: observed.direction,
        native_revision: (*observed.native_revision_for_test()),
        comparison_work_limit: (*observed.comparison_work_limit_for_test()),
        endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(Vec::new()), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
    };
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("insert-matching-path").push(MutationIntent::Create(
                    CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: primary_kind,
                        client_key: ClientKey::raw("new-primary-path"),
                        source: EntityReference::Existing(account.entity_id_for_test()),
                        target: EntityReference::Existing(primary_target),
                        fields: AspectFieldPatch::default(),
                    }),
                )),
            )
        })
        .expect("the fixture read owns its installed host call");
    let current = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(!graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    fact.source_currentness_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        current.application_basis_for_test().snapshot_handle(),
                    )
                    .unwrap()
                    .unwrap()
                    .movement()
                        == Movement::Unmoved
                }))
        })
        .expect("the fixture read owns its installed host call");
    let matching = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    current
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(matching.rows().len(), 1);
    assert_ne!(
        empty.result_set_observation().idempotency_identity_for_test(),
        matching.result_set_observation().idempotency_identity_for_test()
    );
}
