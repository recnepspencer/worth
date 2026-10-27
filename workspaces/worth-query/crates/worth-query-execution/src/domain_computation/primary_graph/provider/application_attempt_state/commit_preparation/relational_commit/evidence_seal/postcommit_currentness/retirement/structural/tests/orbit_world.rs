//! The installed orbit world's real Native reads and mutation publications.
use super::*;

pub(super) fn capture(world: &World, key: u64, execution: &ExecutionRequest<'_, '_>) -> Vec<Fact> {
    let member = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(NodeKey::reference(), key)
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
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let kind = graph
        .layout
        .entity_kind(NodeKey::reference::<OrbitSchema<true>>().entity())
        .unwrap();
    let relation = graph
        .layout
        .relation(Next::reference::<OrbitSchema<true>>().name())
        .unwrap()
        .kind;
    graph
        .integration_handle()
        .with_runtime(execution, |runtime| {
            let pin = runtime
                .read_truth()
                .pin_read_capability(
                    selected.application_basis().snapshot_handle(),
                    &[],
                    execution,
                )
                .unwrap();
            let aggregate = AggregateObservation::read(
                &pin,
                execution,
                AggregateQuery {
                    relation: "Next",
                    entity_kind: kind,
                    relation_kind: relation,
                    member,
                    maximum_work: None,
                    control: None,
                },
            )
            .unwrap();
            let (walk, _) = execution
                .run_native_with_checkpoint(|context| {
                    StructuralWalkObservation::read(
                        &pin,
                        execution,
                        StructuralWalkQuery {
                            relation: "Next",
                            roots: &[member],
                            entity_kind: kind,
                            relation_kind: relation,
                            direction: worth_foundational::facade::RelationDirection::Forward,
                            maximum_work: None,
                            control: None,
                        },
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

pub(super) fn at_current<R>(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    read: impl FnOnce(&RelationalRuntime, &SnapshotHandle) -> R,
) -> R {
    let selected = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            read(runtime, selected.application_basis().snapshot_handle())
        })
}

pub(super) fn nodes(facts: &[Fact]) -> BTreeSet<EntityId> {
    let Fact::Aggregate { observation } = &facts[0] else {
        unreachable!()
    };
    observation.native().members().collect()
}

pub(super) fn walk(facts: &[Fact]) -> &StructuralWalkObservation {
    let Fact::StructuralWalk { observation } = &facts[1] else {
        unreachable!()
    };
    observation
}

pub(super) fn delete_group(world: &World, execution: &ExecutionRequest<'_, '_>, facts: &[Fact]) {
    let mut batch = WorkerIntentBatch::new("whole-orbit-retirement");
    for edge in walk(facts).native().relations() {
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
    fixture::publish_relational_mutation_on_application(execution, &world.application, batch);
}

pub(super) fn replacement(kind: KindId, from: EntityId, to: EntityId) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: from.partition_id,
        kind_id: kind,
        client_key: ClientKey::raw("structural-retirement-twin"),
        source: EntityReference::Existing(from),
        target: EntityReference::Existing(to),
        fields: AspectFieldPatch::default(),
    }))
}
