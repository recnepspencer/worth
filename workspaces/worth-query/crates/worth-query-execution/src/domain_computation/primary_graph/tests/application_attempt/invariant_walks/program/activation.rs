//! This action touches only the root; deep entities cannot enter as touches.
use super::super::{schema::*, world::World};
use super::authorization::authorize;
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in super::super) fn set_active(
    world: &World,
    request: &WorthQueryRequestScope,
    active: bool,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    set_active_on_branch(world, request, active, None)
}

pub(in super::super) fn set_active_on_branch(
    world: &World,
    request: &WorthQueryRequestScope,
    active: bool,
    branch: Option<crate::basis::WorthQueryProductBranch>,
) -> WorthQueryApplicationEffectProgram<WalkSchema, SetActive, SetActiveInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, root) = authorize(
                world,
                &phase,
                &execution,
                request,
                0,
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
