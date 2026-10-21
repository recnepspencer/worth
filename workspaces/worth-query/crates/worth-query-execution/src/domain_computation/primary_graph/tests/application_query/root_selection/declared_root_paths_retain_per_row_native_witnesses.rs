//! declared root paths retain per row native witnesses.
mod join_interruption;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenialKind, WorthQuerySourceExpectationDenialKind,
};

#[test]
fn declared_root_paths_retain_per_row_native_witnesses() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = super::super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
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
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
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
    assert_eq!(query.read_graph().maximum_traversal_depth(), 1);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
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
    assert_eq!(result.receipt().adjacency_list_read_count(), 3);
    assert_eq!(result.receipt().edge_scan_count(), 4);
    assert_eq!(result.receipt().examined_candidate_count(), 3);
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
    assert_eq!(first.aspects.len(), 1);
    assert_eq!(first.adjacencies.len(), 1);
    assert_eq!(second.adjacencies.len(), 1);
    assert!(!first
        .entities
        .contains(&result.observed_sources()[1].footprint_for_test().root));
    assert!(!second
        .entities
        .contains(&result.observed_sources()[0].footprint_for_test().root));
    assert_eq!(
        first.adjacencies[0].relation_kind,
        second.adjacencies[0].relation_kind
    );

    let result = result.into_admitted_disclosed().into_output_demand_source();
    let dependencies = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request())
                .retain_query_dependencies(
                    &phase,
                    &crate::publication_boundary::program_publication_access(),
                    &result,
                )
                .expect("the complete sealed pairing is retained")
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let serial_retained = dependencies.assert_storage_custody_for_test(
                phase
                    .checked_execution_request_for(&world.application.product_runtime)
                    .unwrap(),
            );
            selected
                .require_current_required_set(
                    &phase,
                    &crate::publication_boundary::program_publication_access(),
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
                    &crate::publication_boundary::program_publication_access(),
                    &request,
                    std::iter::empty(),
                    &dependencies,
                )
                .unwrap();
            selected
                .require_current_required_set(
                    &phase,
                    &crate::publication_boundary::program_publication_access(),
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
        &crate::publication_boundary::program_publication_access(),
    );
    let graph = world.application.runtime.primary_graph().unwrap();
    let observed_source = &result.observed_sources()[0];
    let crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity::Product(product) = &observed_source.selection else {
        panic!("the product query must retain its selected branch identity");
    };
    let source_facts = observed_source
        .clone()
        .validate_and_into_facts(
            observed_source.runtime_authority,
            &observed_source.schema_binding,
            &observed_source.branch,
            observed_source.model_root,
            product,
            &observed_source.query_identifier,
            &observed_source.query_identity,
            &graph.layout,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
            if *entity_id == account.entity_id()
    )));
    let kind = first.adjacencies[0].relation_kind;
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let (relation_id, target) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    runtime
                        .read_truth()
                        .visible_relations_of_kind(
                            &host_custody.borrowed_request_for_test(),
                            kind,
                            selected.application_basis().snapshot_handle().version_id(),
                        )
                        .expect("fixture read request remains live")
                        .into_iter()
                        .find(|relation| relation.source == account.entity_id())
                        .map(|relation| (relation.relation_id, relation.target))
                        .unwrap()
                })
        })
        .expect("the fixture read owns its installed host call");
    drop(selected);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::super::super::fixture::publish_relational_mutation(
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
                            source: EntityReference::Existing(account.entity_id()),
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
        .find(|adjacency| adjacency.relation_kind == kind)
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: observed.relation_kind,
        anchor: observed.anchor,
        direction: observed.direction,
        native_revision: observed.native_revision,
        comparison_work_limit: observed.comparison_work_limit,
        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&[target], worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
    };
    let current = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(
                !graph
                    .integration_handle()
                    .with_runtime(request_execution, |runtime| {
                        fact.source_currentness_in(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            current.application_basis().snapshot_handle(),
                        )
                        .unwrap()
                        .unwrap()
                        .movement()
                            == Movement::Unmoved
                    }),
                "same endpoints after relation ABA must stale even a merged adjacency fact"
            )
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| {
        let capture = current.retain_query_dependencies(&phase, &crate::publication_boundary::program_publication_access(), &result).err().unwrap();
        assert!(matches!(capture, crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial::Source(ref cause)
            if cause.kind() == WorthQuerySourceExpectationDenialKind::ForeignOperation));
        let denial = current.require_current_required_set(&phase, &crate::publication_boundary::program_publication_access(), &request,
            std::iter::empty(), &dependencies).unwrap_err();
        assert!(matches!(denial, crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial::Output(ref cause)
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
