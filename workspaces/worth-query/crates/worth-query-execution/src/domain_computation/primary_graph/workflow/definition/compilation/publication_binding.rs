use std::sync::Arc;

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::VersionId;

use super::plan::{
    CompiledWorkflowConnection, CompiledWorkflowDefinition, CompiledWorkflowNode,
    CompiledWorkflowSemanticConnection, CompiledWorkflowSemanticPlan,
};

pub(super) struct ColdCompiledWorkflowDefinition {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) start_node: EntityId,
    pub(super) nodes: Box<[CompiledWorkflowNode]>,
    pub(super) connections: Box<[CompiledWorkflowConnection]>,
    pub(super) revisions: WorkflowDefinitionPublicationRevisions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorkflowDefinitionPublicationRevisions {
    pub(super) start: Option<VersionId>,
    pub(super) nodes: Option<VersionId>,
    pub(super) connections: Option<VersionId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorkflowDefinitionPublicationBinding {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) start_node: EntityId,
    pub(super) node_entities: Box<[EntityId]>,
    pub(super) connection_entities: Box<[EntityId]>,
    pub(super) revisions: WorkflowDefinitionPublicationRevisions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkflowDefinitionPublicationBindingDenial {
    MissingStartNode,
    UnknownConnectionEndpoint,
    BindingCardinalityMismatch,
}

impl WorkflowDefinitionPublicationBinding {
    pub(super) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(std::mem::size_of_val(self.node_entities.as_ref()))
            .saturating_add(std::mem::size_of_val(self.connection_entities.as_ref()))
    }

    pub(super) fn bind(
        &self,
        semantic: Arc<CompiledWorkflowSemanticPlan>,
        content_identity: worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    ) -> Result<CompiledWorkflowDefinition, WorkflowDefinitionPublicationBindingDenial> {
        self.validate_semantic(&semantic)?;
        let nodes = self
            .node_entities
            .iter()
            .zip(semantic.nodes.iter())
            .map(|(entity, meaning)| CompiledWorkflowNode {
                entity: *entity,
                meaning: Arc::clone(meaning),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let connections = self
            .connection_entities
            .iter()
            .zip(semantic.connections.iter())
            .map(|(entity, meaning)| CompiledWorkflowConnection {
                entity: *entity,
                source: self.node_entities[meaning.source],
                target: self.node_entities[meaning.target],
                kind: Arc::clone(&meaning.kind),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(CompiledWorkflowDefinition {
            lineage: self.lineage,
            definition: self.definition,
            content_identity,
            program_revision,
            start_node: self.start_node,
            nodes,
            connections,
        })
    }

    fn validate_semantic(
        &self,
        semantic: &CompiledWorkflowSemanticPlan,
    ) -> Result<(), WorkflowDefinitionPublicationBindingDenial> {
        if self.node_entities.len() != semantic.nodes.len()
            || self.connection_entities.len() != semantic.connections.len()
            || semantic.start >= self.node_entities.len()
            || self.node_entities[semantic.start] != self.start_node
        {
            return Err(WorkflowDefinitionPublicationBindingDenial::BindingCardinalityMismatch);
        }
        if semantic.connections.iter().any(|connection| {
            connection.source >= self.node_entities.len()
                || connection.target >= self.node_entities.len()
        }) {
            return Err(WorkflowDefinitionPublicationBindingDenial::UnknownConnectionEndpoint);
        }
        Ok(())
    }
}

pub(super) fn separate_compiled_definition(
    compiled: ColdCompiledWorkflowDefinition,
) -> Result<
    (
        Arc<CompiledWorkflowSemanticPlan>,
        WorkflowDefinitionPublicationBinding,
    ),
    WorkflowDefinitionPublicationBindingDenial,
> {
    let mut node_order = (0..compiled.nodes.len()).collect::<Vec<_>>();
    node_order.sort_unstable_by(|left, right| {
        compiled.nodes[*left]
            .path()
            .cmp(compiled.nodes[*right].path())
    });
    let node_entities = node_order
        .iter()
        .map(|index| compiled.nodes[*index].entity)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let start = node_entities
        .iter()
        .position(|entity| *entity == compiled.start_node)
        .ok_or(WorkflowDefinitionPublicationBindingDenial::MissingStartNode)?;
    let node_positions = node_entities
        .iter()
        .enumerate()
        .map(|(index, entity)| (*entity, index))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut ordered_connections = compiled
        .connections
        .iter()
        .map(|connection| {
            let source = *node_positions
                .get(&connection.source)
                .ok_or(WorkflowDefinitionPublicationBindingDenial::UnknownConnectionEndpoint)?;
            let target = *node_positions
                .get(&connection.target)
                .ok_or(WorkflowDefinitionPublicationBindingDenial::UnknownConnectionEndpoint)?;
            let identity = format!(
                "{source}:{target}:{}",
                connection.kind.semantic_identity_material()
            );
            Ok((
                identity,
                connection.entity,
                CompiledWorkflowSemanticConnection {
                    source,
                    target,
                    kind: Arc::clone(&connection.kind),
                },
            ))
        })
        .collect::<Result<Vec<_>, WorkflowDefinitionPublicationBindingDenial>>()?;
    ordered_connections.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    let connection_entities = ordered_connections
        .iter()
        .map(|(_, entity, _)| *entity)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let semantic_connections = ordered_connections
        .into_iter()
        .map(|(_, _, connection)| connection)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let semantic_nodes = node_order
        .iter()
        .map(|index| Arc::clone(&compiled.nodes[*index].meaning))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let retained_bytes = semantic_retained_bytes(&semantic_nodes, &semantic_connections);
    let semantic = Arc::new(CompiledWorkflowSemanticPlan {
        start,
        nodes: semantic_nodes,
        connections: semantic_connections,
        retained_bytes,
    });
    let binding = WorkflowDefinitionPublicationBinding {
        lineage: compiled.lineage,
        definition: compiled.definition,
        start_node: compiled.start_node,
        node_entities,
        connection_entities,
        revisions: compiled.revisions,
    };
    Ok((semantic, binding))
}

fn semantic_retained_bytes(
    nodes: &[Arc<super::plan::CompiledWorkflowNodeMeaning>],
    connections: &[CompiledWorkflowSemanticConnection],
) -> usize {
    let node_bytes = nodes.iter().fold(0usize, |total, node| {
        total
            .saturating_add(std::mem::size_of_val(node.as_ref()))
            .saturating_add(node.path.len())
            .saturating_add(node.kind.retained_string_bytes())
    });
    std::mem::size_of::<CompiledWorkflowSemanticPlan>()
        .saturating_add(node_bytes)
        .saturating_add(std::mem::size_of_val(connections))
        .saturating_add(
            connections
                .iter()
                .map(|connection| connection.kind.retained_string_bytes())
                .sum::<usize>(),
        )
}

#[cfg(test)]
mod tests {
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
        assert_eq!(first_semantic.start, 0);
        assert_eq!(foreign_semantic.start, 0);
    }

    #[test]
    fn binding_cardinality_mismatch_is_denied_instead_of_truncated() {
        let (semantic, mut binding) = separate_compiled_definition(cold_definition(10, 20, false))
            .expect("fixture definition must separate");
        binding.node_entities = Box::default();

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
        ColdCompiledWorkflowDefinition {
            lineage: entity(1),
            definition: entity(definition_slot),
            start_node: entity(node_slot),
            nodes: nodes.into_boxed_slice(),
            connections: vec![CompiledWorkflowConnection {
                entity: entity(definition_slot + 1),
                source: entity(node_slot),
                target: entity(node_slot + 1),
                kind: Arc::new(CompiledWorkflowConnectionKind::Control(
                    ApplicationWorkflowControlOutcome::Completed,
                )),
            }]
            .into_boxed_slice(),
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
}
