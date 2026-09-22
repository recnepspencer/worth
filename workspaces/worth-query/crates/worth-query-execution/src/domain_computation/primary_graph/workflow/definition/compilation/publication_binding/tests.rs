use std::sync::Arc;

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::*;
use crate::domain_computation::primary_graph::workflow::definition::compilation::plan::{
    CompiledWorkflowConnectionKind, CompiledWorkflowNodeKind, CompiledWorkflowNodeMeaning,
};

#[test]
fn equal_meaning_normalizes_across_foreign_record_coordinates_and_inventory_order() {
    let first = cold_definition(10, 20, false);
    let foreign = cold_definition(110, 120, true);

    let (first_semantic, first_binding) =
        separate_compiled_definition(first).expect("first definition must separate");
    let (foreign_semantic, foreign_binding) =
        separate_compiled_definition(foreign).expect("foreign definition must separate");

    assert_eq!(first_semantic, foreign_semantic);
    assert_ne!(first_binding.definition, foreign_binding.definition);
    assert_ne!(first_binding.node_entities, foreign_binding.node_entities);
    assert_ne!(first_binding.node_ordinals, foreign_binding.node_ordinals);
    assert_eq!(first_binding.node_ordinals[0], (entity(20), 0));
    assert_eq!(foreign_binding.node_ordinals[0], (entity(120), 0));
    assert_eq!(first_semantic.start, 0);
    assert_eq!(foreign_semantic.start, 0);

    let warm_binding = first_binding.clone();
    assert!(Arc::ptr_eq(
        &first_binding.node_entities,
        &warm_binding.node_entities
    ));
    assert!(Arc::ptr_eq(
        &first_binding.node_ordinals,
        &warm_binding.node_ordinals
    ));
    assert!(Arc::ptr_eq(
        &first_binding.connection_entities,
        &warm_binding.connection_entities
    ));
    let first_compiled = first_binding
        .bound_publication(Arc::clone(&first_semantic))
        .expect("first publication must bind");
    let first_warm = first_binding
        .bound_publication(Arc::clone(&first_semantic))
        .expect("first publication must reuse its binding");
    let foreign_compiled = foreign_binding
        .bound_publication(Arc::clone(&first_semantic))
        .expect("foreign publication must bind shared semantics");
    assert!(Arc::ptr_eq(&first_compiled, &first_warm));
    assert!(Arc::ptr_eq(
        &first_compiled.dispatch,
        &foreign_compiled.dispatch
    ));
    assert_ne!(
        first_compiled.nodes[0].entity,
        foreign_compiled.nodes[0].entity
    );
}

#[test]
fn binding_cardinality_mismatch_is_denied_instead_of_truncated() {
    let (semantic, mut binding) = separate_compiled_definition(cold_definition(10, 20, false))
        .expect("fixture definition must separate");
    binding.node_entities = Arc::from([]);

    assert_eq!(
        binding.validate_semantic(&semantic),
        Err(WorkflowDefinitionPublicationBindingDenial::BindingCardinalityMismatch)
    );
}

#[test]
fn stale_publication_node_lookup_is_denied() {
    let (semantic, mut binding) = separate_compiled_definition(cold_definition(10, 20, false))
        .expect("fixture definition must separate");
    Arc::make_mut(&mut binding.node_ordinals)[0].1 = 1;

    assert_eq!(
        binding.validate_semantic(&semantic),
        Err(WorkflowDefinitionPublicationBindingDenial::BindingCardinalityMismatch)
    );
}

fn cold_definition(
    definition_slot: u64,
    node_slot: u64,
    reverse_inventory: bool,
) -> ColdCompiledWorkflowDefinition {
    let start = node(node_slot, "a/start");
    let terminal = node(node_slot + 1, "z/terminal");
    let mut nodes = vec![start, terminal];
    if reverse_inventory {
        nodes.reverse();
    }
    let mut connections = vec![
        CompiledWorkflowConnection {
            entity: entity(definition_slot + 1),
            source: entity(node_slot),
            target: entity(node_slot + 1),
            kind: Arc::new(CompiledWorkflowConnectionKind::Control(
                ApplicationWorkflowControlOutcome::Completed,
            )),
        },
        CompiledWorkflowConnection {
            entity: entity(definition_slot + 2),
            source: entity(node_slot + 1),
            target: entity(node_slot),
            kind: Arc::new(CompiledWorkflowConnectionKind::Control(
                ApplicationWorkflowControlOutcome::Rejected,
            )),
        },
    ];
    if reverse_inventory {
        connections.reverse();
    }
    ColdCompiledWorkflowDefinition {
        lineage: entity(1),
        definition: entity(definition_slot),
        start_node: entity(node_slot),
        nodes: nodes.into_boxed_slice(),
        connections: connections.into_boxed_slice(),
        revisions: WorkflowDefinitionPublicationRevisions {
            start: None,
            nodes: None,
            connections: None,
        },
    }
}

fn node(slot: u64, path: &str) -> CompiledWorkflowNode {
    CompiledWorkflowNode {
        entity: entity(slot),
        meaning: Arc::new(CompiledWorkflowNodeMeaning {
            path: path.to_owned(),
            kind: CompiledWorkflowNodeKind::Terminal,
        }),
    }
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(1), slot, 1)
}
