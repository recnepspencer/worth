//! Genuine Query adjacency fact observation through native pinned admission.
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::{
    fact::{WorthQueryApplicationFactKey as Key, WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationAdjacencyDirection as Direction},
    read_set::fact_observation::observe_fact,
};
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::{
    decision::identity, program::authorize, schema::{Node, ReplaceNext},
};

#[test]
fn query_adjacency_fact_observation_builds_no_schema_projection() {
    let world = identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let (_, projections) = RelationalSchemaRegistry::count_authority_projections_for_test(|| {
        world
            .application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request_for_test();
                let execution = host
                    .execution_request_for_for_test(world.application.product_runtime())
                    .unwrap();
                let (admission, _) = authorize(
                    &world,
                    &phase,
                    &execution,
                    &request,
                    0,
                    ReplaceNext::reference(),
                );
                let (_, projection, _) = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, root| {
                            reader
                                .require_decision_entity(root, Node::reference())
                                .unwrap();
                        },
                        Allocation::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let attempt = world
                    .application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        Allocation::SystemAllocation,
                    )
                    .unwrap();
                let anchor = (*attempt.expected_facts_for_test())
                    .as_ref()
                    .unwrap()
                    .iter_for_test()
                    .find_map(|key| match key {
                        Key::Entity { entity, entity_id } if entity == "Node" => Some(*entity_id),
                        _ => None,
                    })
                    .expect("genuinely observed root");
                (*attempt.lease_for_test())
                    .handle_for_test()
                    .with_runtime(&execution, |runtime| {
                        let key = Key::Adjacency {
                            relation: "Next".into(),
                            anchor,
                            direction: Direction::Outgoing,
                            maximum_work_units: 100,
                        };
                        let fact = observe_fact(
                            &execution,
                            runtime,
                            (*attempt.lease_for_test()).snapshot_for_test(),
                            &(*(*attempt.lease_for_test()).layout_for_test()),
                            &key,
                        )
                        .unwrap();
                        let Fact::Adjacency { relations, .. } = fact else {
                            panic!("actual adjacency fact required")
                        };
                        assert_eq!(
                            relations.len(),
                            1,
                            "the published orbit has one physical successor"
                        );
                    });
            })
            .unwrap();
    });
    assert_eq!(
        projections, 0,
        "Query snapshot admission or fact observation reconstructed schema authority"
    );
}
