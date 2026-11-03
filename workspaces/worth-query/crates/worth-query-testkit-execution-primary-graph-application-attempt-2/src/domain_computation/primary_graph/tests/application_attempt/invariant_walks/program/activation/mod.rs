//! This action touches only the root; deep entities cannot enter as touches.
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::World;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::authorization::authorize;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub fn set_active(
    world: &World,
    request: &WorthQueryRequestScope,
    active: bool,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    set_active_on_branch(world, request, active, None)
}

pub fn set_active_on_branch(
    world: &World,
    request: &WorthQueryRequestScope,
    active: bool,
    branch: Option<worth_query_execution::facade::product::WorthQueryProductBranch>,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    set_node_on_branch(world, request, 0, active, branch)
}
pub fn set_node_active(
    world: &World,
    request: &WorthQueryRequestScope,
    node: u64,
    active: bool,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    set_node_on_branch(world, request, node, active, None)
}
pub fn set_node_on_branch(
    world: &World,
    request: &WorthQueryRequestScope,
    node: u64,
    active: bool,
    branch: Option<worth_query_execution::facade::product::WorthQueryProductBranch>,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            let (admission, root) = authorize(
                world,
                &phase,
                &execution,
                request,
                node,
                SetActive::reference(),
                branch,
            );
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, subject| {
                        reader
                            .require_decision_entity(subject, Node::reference())
                            .unwrap();
                        reader
                            .require_decision_field(subject, Active::reference())
                            .unwrap();
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let root = effects.existing_entity(&root).unwrap();
            effects
                .write_field(&root, Active::reference(), active)
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}
