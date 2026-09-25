use std::sync::{Arc, OnceLock};

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::VersionId;

use super::plan::{
    CompiledWorkflowConnection, CompiledWorkflowDefinition, CompiledWorkflowNode,
    CompiledWorkflowPublicationPlan, CompiledWorkflowSemanticConnection,
    CompiledWorkflowSemanticPlan,
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

#[derive(Clone)]
pub(super) struct WorkflowDefinitionPublicationBinding {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) start_node: EntityId,
    pub(super) node_entities: Arc<[EntityId]>,
    pub(super) node_ordinals: Arc<[(EntityId, usize)]>,
    pub(super) connection_entities: Arc<[EntityId]>,
    pub(super) revisions: WorkflowDefinitionPublicationRevisions,
    compiled: Arc<OnceLock<Arc<CompiledWorkflowPublicationPlan>>>,
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
            .saturating_add(std::mem::size_of_val(self.node_ordinals.as_ref()))
            .saturating_add(std::mem::size_of_val(self.connection_entities.as_ref()))
            .saturating_add(std::mem::size_of::<
                OnceLock<Arc<CompiledWorkflowPublicationPlan>>,
            >())
            .saturating_add(std::mem::size_of::<CompiledWorkflowPublicationPlan>())
            .saturating_add(
                self.node_entities
                    .len()
                    .saturating_mul(std::mem::size_of::<CompiledWorkflowNode>()),
            )
            .saturating_add(
                self.connection_entities
                    .len()
                    .saturating_mul(std::mem::size_of::<CompiledWorkflowConnection>()),
            )
    }

    pub(super) fn bind(
        &self,
        semantic: Arc<CompiledWorkflowSemanticPlan>,
        content_identity: worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    ) -> Result<CompiledWorkflowDefinition, WorkflowDefinitionPublicationBindingDenial> {
        let publication = self.bound_publication(semantic)?;
        Ok(CompiledWorkflowDefinition {
            publication,
            content_identity,
            program_revision,
        })
    }

    fn bound_publication(
        &self,
        semantic: Arc<CompiledWorkflowSemanticPlan>,
    ) -> Result<Arc<CompiledWorkflowPublicationPlan>, WorkflowDefinitionPublicationBindingDenial>
    {
        let publication = if let Some(compiled) = self.compiled.get() {
            Arc::clone(compiled)
        } else {
            self.validate_semantic(&semantic)?;
            Arc::clone(
                self.compiled
                    .get_or_init(|| Arc::new(self.compile_publication(semantic.as_ref()))),
            )
        };
        Ok(publication)
    }

    fn compile_publication(
        &self,
        semantic: &CompiledWorkflowSemanticPlan,
    ) -> CompiledWorkflowPublicationPlan {
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
        CompiledWorkflowPublicationPlan {
            lineage: self.lineage,
            definition: self.definition,
            start: semantic.start,
            nodes,
            connections,
            node_ordinals: Arc::clone(&self.node_ordinals),
            dispatch: Arc::clone(&semantic.dispatch),
        }
    }

    fn validate_semantic(
        &self,
        semantic: &CompiledWorkflowSemanticPlan,
    ) -> Result<(), WorkflowDefinitionPublicationBindingDenial> {
        if self.node_entities.len() != semantic.nodes.len()
            || self.node_ordinals.len() != semantic.nodes.len()
            || self.connection_entities.len() != semantic.connections.len()
            || semantic.start >= self.node_entities.len()
            || self.node_entities[semantic.start] != self.start_node
        {
            return Err(WorkflowDefinitionPublicationBindingDenial::BindingCardinalityMismatch);
        }
        if self
            .node_ordinals
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0)
            || self
                .node_ordinals
                .iter()
                .any(|(entity, ordinal)| self.node_entities.get(*ordinal) != Some(entity))
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
    ordered_connections
        .sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
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
    let dispatch = Arc::new(super::plan::CompiledWorkflowDispatch::build(
        semantic_nodes.len(),
        &semantic_connections,
    ));
    let retained_bytes = semantic_retained_bytes(&semantic_nodes, &semantic_connections, &dispatch);
    let semantic = Arc::new(CompiledWorkflowSemanticPlan {
        start,
        nodes: semantic_nodes,
        connections: semantic_connections,
        dispatch,
        retained_bytes,
    });
    let mut node_ordinals = node_entities
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, entity)| (entity, ordinal))
        .collect::<Vec<_>>();
    node_ordinals.sort_unstable_by_key(|(entity, _)| *entity);
    let binding = WorkflowDefinitionPublicationBinding {
        lineage: compiled.lineage,
        definition: compiled.definition,
        start_node: compiled.start_node,
        node_entities: Arc::from(node_entities),
        node_ordinals: Arc::from(node_ordinals),
        connection_entities: Arc::from(connection_entities),
        revisions: compiled.revisions,
        compiled: Arc::new(OnceLock::new()),
    };
    Ok((semantic, binding))
}

fn semantic_retained_bytes(
    nodes: &[Arc<super::plan::CompiledWorkflowNodeMeaning>],
    connections: &[CompiledWorkflowSemanticConnection],
    dispatch: &super::plan::CompiledWorkflowDispatch,
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
        .saturating_add(dispatch.retained_bytes())
        .saturating_add(
            connections
                .iter()
                .map(|connection| connection.kind.retained_string_bytes())
                .sum::<usize>(),
        )
}

#[cfg(test)]
#[path = "publication_binding/tests.rs"]
mod tests;
