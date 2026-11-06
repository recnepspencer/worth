//! declared root paths retain per row native witnesses.
pub use crate::domain_computation::primary_graph::tests::application_query::root_selection::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenialKind;
use worth_query_execution::publication_boundary::program_publication_access;
#[test]
fn declared_root_paths_retain_per_row_native_witnesses() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let (principal, account) =
        crate::domain_computation::primary_graph::tests::application_query::root_selection::declared_root_paths_retain_per_row_native_witnesses::party::alice_and_open_account(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    assert_eq!(query.read_graph().maximum_traversal_depth(), 1);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        1
    );
    let predicate_fields = plan
        .graph_read_plan()
        .requirements()
        .rows()
        .iter()
        .flat_map(|row| row.predicate_field_authorities())
        .map(|field| field.native_field_key().as_str())
        .collect::<Vec<_>>();
    assert_eq!(predicate_fields, ["AccountStatus"]);
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        result
            .rows()
            .iter()
            .map(|row| row.sequence())
            .collect::<Vec<_>>(),
        [11, 22]
    );
    assert_eq!(result.receipt().result_count(), 2);
    assert_eq!(result.receipt().reads().adjacency_lists_read(), 3);
    assert_eq!(result.receipt().reads().adjacency_edges_inspected(), 4);
    assert_eq!(result.receipt().examined_candidate_count(), 3);
    assert!(
        result.receipt().reads().field_reads() > result.receipt().projected_field_count(),
        "path guards compare candidates beyond the returned projection fields"
    );
    assert_eq!(result.receipt().reads().index_candidates_examined(), 0);
    assert_eq!(result.receipt().work().predicate_work_units(), 6);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
    let first = std::sync::Arc::clone(
        result.observed_sources()[0]
            .footprint_for_test()
            .root_selection
            .as_ref()
            .expect("path-selected rows retain native selection evidence"),
    );
    let second = std::sync::Arc::clone(
        result.observed_sources()[1]
            .footprint_for_test()
            .root_selection
            .as_ref()
            .expect("every row retains its own path witness"),
    );
    assert!(!std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!((*first.aspects_for_test()).len(), 1);
    assert_eq!(first.adjacencies.len(), 1);
    assert_eq!(second.adjacencies.len(), 1);
    assert!(!(*first.entities_for_test()).contains(
        result.observed_sources()[1]
            .footprint_for_test()
            .root_for_test()
    ));
    assert!(!(*second.entities_for_test()).contains(
        result.observed_sources()[0]
            .footprint_for_test()
            .root_for_test()
    ));
    assert_eq!(
        (*first.adjacencies[0].relation_kind_for_test()),
        (*second.adjacencies[0].relation_kind_for_test())
    );
    let result = result.into_admitted_disclosed().into_output_demand_source();
    let dependencies = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request_for_test())
                .retain_query_dependencies(&phase, &program_publication_access(), &result)
                .expect("the complete sealed pairing is retained")
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let serial_retained = dependencies.assert_storage_custody_for_test(
                phase
                    .checked_execution_request_for(
                        (*world.application.runtime_for_test()).product_runtime_for_test(),
                    )
                    .unwrap(),
            );
            selected
                .require_current_required_set(
                    &phase,
                    &program_publication_access(),
                    &request,
                    std::iter::empty(),
                    &serial_retained,
                )
                .expect(
                    "the same original proof retained by fresh serial request is actually current",
                );
            selected
                .require_current_required_set(
                    &phase,
                    &program_publication_access(),
                    &request,
                    std::iter::empty(),
                    &dependencies,
                )
                .unwrap();
            selected
                .require_current_required_set(
                    &phase,
                    &program_publication_access(),
                    &request,
                    std::iter::empty(),
                    &dependencies,
                )
                .unwrap();
        })
        .unwrap();
    join_interruption::assert_original_join_cancellation(
        &world,
        &dependencies,
        &program_publication_access(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let observed_source = &result.observed_sources()[0];
    let worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity::Product(product) = observed_source.selection_for_test() else {
        panic!("the product query must retain its selected branch identity");
    };
    let source_facts = observed_source
        .clone()
        .validate_and_into_facts_for_test(
            *observed_source.runtime_authority_for_test(),
            observed_source.schema_binding_for_test(),
            observed_source.branch_for_test(),
            *observed_source.model_root_for_test(),
            product,
            observed_source.query_identifier_for_test(),
            observed_source.query_identity_for_test(),
            graph.layout_for_test(),
            None,
            request_execution,
        )
        .expect("ordinary source binding retains root-path native facts");
    assert_eq!(
        source_facts
            .iter()
            .filter(|fact| matches!(
                fact,
                WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            ))
            .count(),
        1
    );
    assert!(source_facts.iter().any(|fact| matches!(fact,
        WorthQueryApplicationObservedFact::SourceFieldRevision { entity_id, .. }
            if *entity_id == account.entity_id_for_test()
    )));
    let kind = *first.adjacencies[0].relation_kind_for_test();
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let (relation_id, target) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            graph
                .integration_handle_for_test()
                .with_runtime(request_execution, |runtime| {
                    runtime
                        .read_truth()
                        .visible_relations_of_kind(
                            &host_custody.borrowed_request_for_test(),
                            kind,
                            selected
                                .application_basis_for_test()
                                .snapshot_handle()
                                .version_id(),
                        )
                        .expect("fixture read request remains live")
                        .into_iter()
                        .find(|relation| relation.source == account.entity_id_for_test())
                        .map(|relation| (relation.relation_id, relation.target))
                        .unwrap()
                })
        })
        .expect("the fixture read owns its installed host call");
    drop(selected);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
                &host_custody.borrowed_request_for_test(),
                &world,
                WorkerIntentBatch::new("root-path-relation-aba")
                    .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                        DeleteRelationIntent { relation_id },
                    )))
                    .push(MutationIntent::Create(CreateIntent::Relation(
                        RelationSpec {
                            partition_id: PartitionId::main(),
                            kind_id: kind,
                            client_key: ClientKey::raw("root-path-relation-aba-replacement"),
                            source: EntityReference::Existing(account.entity_id_for_test()),
                            target: EntityReference::Existing(target),
                            fields: AspectFieldPatch::default(),
                        },
                    ))),
            )
        })
        .expect("the fixture read owns its installed host call");
    let observed = first
        .adjacencies
        .iter()
        .find(|adjacency| (*adjacency.relation_kind_for_test()) == kind)
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: (*observed.relation_kind_for_test()),
        anchor: (*observed.anchor_for_test()),
        direction: observed.direction,
        native_revision: (*observed.native_revision_for_test()),
        comparison_work_limit: (*observed.comparison_work_limit_for_test()),
        endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&[target], worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
    };
    let current = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(
                !graph
                    .integration_handle_for_test()
                    .with_runtime(request_execution, |runtime| {
                        fact.source_currentness_in_for_test(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            current.application_basis_for_test().snapshot_handle(),
                        )
                        .unwrap()
                        .unwrap()
                        .movement_for_test()
                            == Movement::Unmoved
                    }),
                "same endpoints after relation ABA must stale even a merged adjacency fact"
            )
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| {
        let capture = current.retain_query_dependencies(&phase, &program_publication_access(), &result).err().unwrap();
        assert!(matches!(capture, worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial::Source(ref cause)
            if cause.kind() == WorthQuerySourceExpectationDenialKind::ForeignOperation));
        let denial = current.require_current_required_set(&phase, &program_publication_access(), &request,
            std::iter::empty(), &dependencies).unwrap_err();
        assert!(matches!(denial, worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial::Output(ref cause)
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded), "real native adjacency ABA refuses the complete original pairing: {denial:?}");
    }).unwrap();
    let refreshed = world
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
    assert_eq!(
        refreshed
            .rows()
            .iter()
            .map(|row| row.sequence())
            .collect::<Vec<_>>(),
        [11, 22],
        "ABA keeps the public result set unchanged"
    );
    assert_ne!(
        result.observed_sources()[0].idempotency_identity(),
        refreshed.observed_sources()[0].idempotency_identity(),
        "native path revisions must participate in source identity"
    );
    assert_ne!(
        result.observed_sources()[1].idempotency_identity(),
        refreshed.observed_sources()[1].idempotency_identity(),
        "rows reached through the same native adjacency granule both become stale"
    );
}
#[path = "join_interruption/mod.rs"]
pub mod join_interruption;
#[path = "party/mod.rs"]
pub mod party;
