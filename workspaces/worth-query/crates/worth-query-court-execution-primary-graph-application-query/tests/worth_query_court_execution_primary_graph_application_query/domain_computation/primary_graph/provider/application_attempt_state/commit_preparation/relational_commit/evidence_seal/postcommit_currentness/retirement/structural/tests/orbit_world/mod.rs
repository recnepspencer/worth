//! The installed orbit world's real Native reads and mutation publications.
use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::structural::tests::*;

pub fn capture(world: &World, key: u64, execution: &ExecutionRequest<'_, '_>) -> Vec<Fact> {
    let member = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(NodeKey::at("NodeKey"), key)
                            .unwrap()
                            .entity_id()
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let selected = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let kind = (*graph.layout_for_test())
        .entity_kind_for_test(NodeKey::at("NodeKey").entity())
        .unwrap();
    let relation = *(*graph.layout_for_test())
        .relation(Next::reference::<OrbitSchema<true>>().name())
        .unwrap()
        .kind_for_test();
    graph
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            let pin = runtime
                .read_truth()
                .pin_read_capability(
                    selected.application_basis_for_test().snapshot_handle(),
                    &[],
                    execution,
                )
                .unwrap();
            let aggregate = AggregateObservation::read_for_test(
                &pin,
                execution,
                AggregateQuery {
                    allocation: None,
                    subject: "Next",
                    entity_kind: kind,
                    selection: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation::selection::AggregateSelection::OrbitRelation(relation),
                    member,
                    maximum_work: None,
                    control: None,
                },
            )
            .unwrap();
            let (walk, _) = execution
                .run_native_with_checkpoint(|context| {
                    StructuralWalkObservation::read_for_test(
                        &pin,
                        execution,
                        StructuralWalkQuery::new_for_test("Next", &[member], kind, relation, worth_foundational::facade::RelationDirection::Forward, None, None),
                        context,
                    )
                    .unwrap()
                })
                .unwrap();
            vec![
                Fact::Aggregate {
                    observation: aggregate,
                },
                Fact::StructuralWalk { observation: walk },
            ]
        })
}

pub fn at_current<R>(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    read: impl FnOnce(&RelationalRuntime, &SnapshotHandle) -> R,
) -> R {
    let selected = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            read(
                runtime,
                selected.application_basis_for_test().snapshot_handle(),
            )
        })
}

pub fn nodes(facts: &[Fact]) -> BTreeSet<EntityId> {
    let Fact::Aggregate { observation } = &facts[0] else {
        unreachable!()
    };
    observation.native().members().collect()
}

pub fn walk(facts: &[Fact]) -> &StructuralWalkObservation {
    let Fact::StructuralWalk { observation } = &facts[1] else {
        unreachable!()
    };
    observation
}

pub fn delete_group(world: &World, execution: &ExecutionRequest<'_, '_>, facts: &[Fact]) {
    let mut batch = WorkerIntentBatch::new("whole-orbit-retirement");
    for edge in walk(facts).native_for_test().relations() {
        batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: edge.relation_id,
            },
        )));
    }
    for entity in nodes(facts) {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: entity },
        )));
    }
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(execution, &world.application, batch);
}

pub fn replacement(kind: KindId, from: EntityId, to: EntityId) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: from.partition_id,
        kind_id: kind,
        client_key: ClientKey::raw("structural-retirement-twin"),
        source: EntityReference::Existing(from),
        target: EntityReference::Existing(to),
        fields: AspectFieldPatch::default(),
    }))
}
