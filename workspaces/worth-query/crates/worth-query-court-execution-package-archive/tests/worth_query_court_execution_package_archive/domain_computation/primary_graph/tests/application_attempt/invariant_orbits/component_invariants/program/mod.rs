//! Ordinary admitted CRUD changes only the initially empty component member kind.
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub fn change_spare<const O: bool>(
    world: &World<O>,
    request: &WorthQueryRequestScope,
    create: bool,
) -> WorthQueryApplicationEffectProgram<OrbitSchema<O>, ReplaceNext, ReplaceNextInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            let (admission, root) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize(
                world,
                &phase,
                &execution,
                request,
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
                        reader
                            .require_decision_field(root, NodeKey::at("NodeKey"))
                            .unwrap();
                        let spare = reader
                            .resolve_optional_entity(SpareKey::at("SpareKey"), 19u64)
                            .unwrap();
                        if create {
                            assert!(spare.is_none());
                        } else {
                            let spare = spare.unwrap();
                            reader
                                .require_decision_entity(&spare, Spare::reference())
                                .unwrap();
                            reader
                                .require_decision_field(&spare, SpareKey::at("SpareKey"))
                                .unwrap();
                        }
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
            let selected = world
                .application
                .select_product_branch_for_test(
                    &execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .unwrap();
            let spare = if create {
                None
            } else {
                Some(
                    selected
                        .resolve_entity(
                            &phase,
                            SpareKey::at("SpareKey"),
                            19u64,
                            request,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap(),
                )
            };
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            if create {
                let root = effects.existing_entity(&root).unwrap();
                let spare = effects
                    .create_entity_in_context(
                        &root,
                        Spare::reference(),
                        WorthQueryApplicationEntityKey::new("component-spare-19").unwrap(),
                    )
                    .unwrap();
                effects
                    .initialize_field(&spare, SpareKey::at("SpareKey"), 19u64)
                    .unwrap();
            } else {
                let spare = effects.existing_entity(&spare.unwrap()).unwrap();
                effects.delete_entity(Spare::reference(), &spare).unwrap();
            }
            effects.finish().unwrap()
        })
        .unwrap()
}
