//! Real declaration validation, complete fact production and native compiler binding.
use super::super::super::publication_binding::{
    separate_compiled_definition, ColdCompiledWorkflowDefinition,
    WorkflowDefinitionPublicationRevisions,
};
use super::*;
use crate::domain_computation::primary_graph::workflow::{
    definition::{visit_definition_facts, WorkflowLineagePublicationTarget},
    schema::WorthQueryWorkflowLayout,
};
use worth_query_declaration::facade::application_program::*;
use worth_relational::facade::identity::PartitionId;

struct ProvenanceWorkflow;
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

pub(super) fn publish_definition(
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
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let inventories = handle.with_runtime(execution, |runtime| {
        [
            layout.lineage.entity_kind,
            layout.definition.entity_kind,
            layout.node.entity_kind,
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
        layout.lineage.entity_kind,
        &inventories[0],
    );
    let entity = only_new_entity(
        world,
        execution,
        layout.definition.entity_kind,
        &inventories[1],
    );
    let node = only_new_entity(world, execution, layout.node.entity_kind, &inventories[2]);
    let retained = selected(world, execution);
    let compiled_node = handle.with_runtime(execution, |runtime| {
        let mut observed = Vec::new();
        super::super::node::compile_node(
            execution,
            runtime,
            retained.1.snapshot_handle(),
            layout,
            node,
            &mut observed,
        )
        .unwrap()
    });
    let (semantic, binding) = separate_compiled_definition(ColdCompiledWorkflowDefinition {
        lineage,
        definition: entity,
        start_node: node,
        nodes: vec![compiled_node].into_boxed_slice(),
        connections: Box::new([]),
        global_actions: Box::new([]),
        revisions: WorkflowDefinitionPublicationRevisions {
            start: None,
            nodes: None,
            connections: None,
        },
    })
    .unwrap();
    binding
        .bind(semantic, definition.content_identity().clone(), revision)
        .unwrap()
}
