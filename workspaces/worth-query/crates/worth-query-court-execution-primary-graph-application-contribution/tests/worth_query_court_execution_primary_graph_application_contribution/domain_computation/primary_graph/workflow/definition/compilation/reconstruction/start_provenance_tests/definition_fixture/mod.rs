//! Real declaration validation, complete fact production and native compiler binding.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::compilation::publication_binding::separate_compiled_definition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::compilation::publication_binding::ColdCompiledWorkflowDefinition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::compilation::publication_binding::WorkflowDefinitionPublicationRevisions;
use crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::facts::visit_definition_facts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::facts::WorkflowLineagePublicationTarget;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::schema::layout::WorthQueryWorkflowLayout;
use worth_query_declaration::facade::application_program::*;
use worth_relational::facade::identity::PartitionId;

pub struct ProvenanceWorkflow;
impl ApplicationWorkflowSpec for ProvenanceWorkflow {
    type Schema = IdentityExecutionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.native-start-provenance.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}

pub fn publish_definition(
    world: &AuthorizationWorld,
    execution: &ExecutionRequest<'_, '_>,
    layout: &WorthQueryWorkflowLayout,
    revision: ApplicationProgramRevision,
    identity: &str,
    partition: PartitionId,
) -> CompiledWorkflowDefinition {
    let limits = ApplicationWorkflowDefinitionLimits::new(
        1,
        1,
        16,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        4096,
    )
    .unwrap();
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ProvenanceWorkflow>::new(identity, limits).unwrap();
    let terminal = builder.terminal("complete").unwrap();
    builder.start(&terminal);
    let definition = builder.finish().unwrap().validate().unwrap();
    let before = selected(world, execution);
    let handle = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test();
    let inventories = handle.with_runtime(execution, |runtime| {
        [
            (*(*layout.lineage_for_test()).entity_kind_for_test()),
            (*(*layout.definition_for_test()).entity_kind_for_test()),
            (*(*layout.node_for_test()).entity_kind_for_test()),
        ]
        .map(|kind| {
            runtime
                .read_truth()
                .visible_entities_of_kind(execution, kind, before.1.snapshot_handle().version_id())
                .unwrap()
                .into_iter()
                .map(|record| record.entity_id)
                .collect::<Vec<_>>()
        })
    });
    let mut effects = Vec::new();
    visit_definition_facts(
        layout,
        &revision,
        &definition,
        &[],
        &[],
        &[],
        WorkflowLineagePublicationTarget::New,
        |effect| {
            effects.push(effect);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    // Includes lineage, definition, terminal, lineage ownership, current,
    // definition-node and required definition-start. No endpoint is omitted.
    assert_eq!(effects.len(), 7);
    publish_effects(world, execution, partition, effects);
    let lineage = only_new_entity(
        world,
        execution,
        *(*layout.lineage_for_test()).entity_kind_for_test(),
        &inventories[0],
    );
    let entity = only_new_entity(
        world,
        execution,
        *(*layout.definition_for_test()).entity_kind_for_test(),
        &inventories[1],
    );
    let node = only_new_entity(
        world,
        execution,
        *(*layout.node_for_test()).entity_kind_for_test(),
        &inventories[2],
    );
    let retained = selected(world, execution);
    let compiled_node = handle.with_runtime(execution, |runtime| {
        let mut observed = Vec::new();
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::node::compile_node(
            execution,
            runtime,
            retained.1.snapshot_handle(),
            layout,
            node,
            &mut observed,
        )
        .unwrap()
    });
    let (semantic, binding) =
        separate_compiled_definition(ColdCompiledWorkflowDefinition::new_for_test(
            lineage,
            entity,
            node,
            vec![compiled_node].into_boxed_slice(),
            Box::new([]),
            Box::new([]),
            WorkflowDefinitionPublicationRevisions::new_for_test(None, None, None),
        ))
        .unwrap();
    binding
        .bind_for_test(semantic, definition.content_identity().clone(), revision)
        .unwrap()
}
