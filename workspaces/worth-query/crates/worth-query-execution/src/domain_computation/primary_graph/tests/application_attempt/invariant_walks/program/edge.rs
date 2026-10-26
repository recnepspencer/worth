//! Replace exactly the observed third-hop edge through ordinary unlink/link.
use super::super::{schema::*, world::World};
use super::authorization::authorize;
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in super::super) fn replace_next(
    world: &World,
    request: &WorthQueryRequestScope,
    target: u64,
) -> WorthQueryApplicationEffectProgram<WalkSchema, ReplaceNext, ReplaceNextInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, from) = authorize(
                world,
                &phase,
                &execution,
                request,
                2,
                ReplaceNext::reference(),
                None,
            );
            let selected = world
                .application
                .select_product_branch(
                    &execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let old = selected
                .resolve_entity(
                    &phase,
                    NodeKey::reference(),
                    3,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let new = selected
                .resolve_entity(
                    &phase,
                    NodeKey::reference(),
                    target,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
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
                            .require_decision_field(subject, NodeKey::reference())
                            .unwrap();
                        let previous = reader.resolve_entity(NodeKey::reference(), 3).unwrap();
                        reader
                            .require_decision_relation(Next::reference(), subject, &previous)
                            .unwrap();
                        let replacement =
                            reader.resolve_entity(NodeKey::reference(), target).unwrap();
                        reader
                            .require_decision_entity(&replacement, Node::reference())
                            .unwrap();
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
            let observed = reads
                .observe_relation(&execution, Next::reference(), &from, &old)
                .unwrap();
            assert_eq!(
                observed.count(),
                1,
                "the actual third-hop edge must exist before editing"
            );
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let from = effects.existing_entity(&from).unwrap();
            let new = effects.existing_entity(&new).unwrap();
            effects.unlink(Next::reference(), observed).unwrap();
            effects
                .link(
                    Next::reference(),
                    format!("replacement-{target}"),
                    &from,
                    &new,
                )
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}
