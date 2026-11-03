//! Add one explicitly admitted absent pair at an existing empty leaf.
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::World;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::authorization::authorize;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::facade::authorization;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub fn link(
    world: &World,
    request: &WorthQueryRequestScope,
    source: u64,
    target: u64,
) -> WorthQueryApplicationEffectProgram<WalkSchema, ReplaceNext, ReplaceNextInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, from) = authorize(
                world,
                &phase,
                &execution,
                request,
                source,
                ReplaceNext::reference(),
                None,
            );
            let selected = world
                .application
                .select_product_branch_for_test(
                    &execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .unwrap();
            let to = selected
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
                        let target = reader.resolve_entity(NodeKey::reference(), target).unwrap();
                        reader
                            .require_decision_entity(&target, Node::reference())
                            .unwrap();
                        reader
                            .require_decision_relation(Next::reference(), subject, &target)
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
            assert_eq!(
                reads
                    .observe_relation(&execution, Next::reference(), &from, &to)
                    .unwrap()
                    .count(),
                0
            );
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let from = effects.existing_entity(&from).unwrap();
            let to = effects.existing_entity(&to).unwrap();
            effects
                .link(
                    Next::reference(),
                    format!("leaf-link-{source}-{target}"),
                    &from,
                    &to,
                )
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}
