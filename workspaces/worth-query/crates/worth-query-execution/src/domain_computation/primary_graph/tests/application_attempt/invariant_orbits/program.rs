//! Swap two observed successors in one ordinary, law-preserving commit proposal.
use super::{schema::*, world::World};
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(super) fn swap_successors<const O: bool>(
    world: &World<O>,
    request: &WorthQueryRequestScope,
    left: u64,
    right: u64,
) -> WorthQueryApplicationEffectProgram<OrbitSchema<O>, ReplaceNext, ReplaceNextInput, Node> {
    let edges = world.edges();
    let pairs =
        [left, right].map(|source| *edges.iter().find(|(from, _)| *from == source).unwrap());
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = authorize(
                world,
                &phase,
                &execution,
                request,
                left,
                ReplaceNext::reference(),
            );
            let selected = world
                .application
                .select_product_branch(
                    &execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let entities = pairs.map(|(from, to)| {
                [from, to].map(|key| {
                    selected
                        .resolve_entity(
                            &phase,
                            NodeKey::reference(),
                            key,
                            request,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap()
                })
            });
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        for (from, to) in pairs {
                            let from = reader.resolve_entity(NodeKey::reference(), from).unwrap();
                            let to = reader.resolve_entity(NodeKey::reference(), to).unwrap();
                            for entity in [&from, &to] {
                                reader
                                    .require_decision_entity(entity, Node::reference())
                                    .unwrap();
                                reader
                                    .require_decision_field(entity, NodeKey::reference())
                                    .unwrap();
                            }
                            reader
                                .require_decision_relation(Next::reference(), &from, &to)
                                .unwrap();
                        }
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let mut observed = Vec::new();
            for [from, to] in &entities {
                let edge = reads
                    .observe_relation(&execution, Next::reference(), from, to)
                    .unwrap();
                assert_eq!(edge.count(), 1, "both original physical edges must exist");
                observed.push(edge);
            }
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let sources = [&entities[0][0], &entities[1][0]]
                .map(|entity| effects.existing_entity(entity).unwrap());
            let targets = [&entities[1][1], &entities[0][1]]
                .map(|entity| effects.existing_entity(entity).unwrap());
            for edge in observed {
                effects.unlink(Next::reference(), edge).unwrap();
            }
            for (index, (source, target)) in sources.iter().zip(&targets).enumerate() {
                effects
                    .link(
                        Next::reference(),
                        format!("swap-{left}-{right}-{index}"),
                        source,
                        target,
                    )
                    .unwrap();
            }
            effects.finish().unwrap()
        })
        .unwrap()
}

use worth_query_declaration::facade::application_schema::*;
fn authorize<const O: bool, Operation, Input>(
    world: &World<O>,
    phase: &WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    key: u64,
    operation: ApplicationOperationRef<OrbitSchema<O>, Operation, Input>,
) -> (
    WorthQueryAdmittedApplicationOperation<OrbitSchema<O>, Operation, Input, Node>,
    WorthQueryApplicationEntityIdentity<OrbitSchema<O>, Node>,
)
where
    Operation: ApplicationOperationMarkerIdentity<OrbitSchema<O>> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: 'static,
{
    let selected = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let external = crate::domain_computation::primary_graph::tests::fixture::authenticate_external(
        world.application.installed_schema(),
        "alice",
        std::time::Duration::from_secs(60),
        request,
    );
    let actor = selected
        .resolve_authenticated_principal(
            phase,
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let subject = selected
        .resolve_entity(
            phase,
            NodeKey::reference(),
            key,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(operation)
        .unwrap();
    let admission = selected
        .authorize_operation(
            phase,
            &actor,
            &subject,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    (admission, subject)
}
