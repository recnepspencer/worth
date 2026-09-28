//! What one published definition depends on, read from its retained facts.
//!
//! Program adoption compares these facts with installed vocabulary coverage.
//! It never reconstructs a compiled plan, so a definition the target cannot
//! execute is still inventoried exactly and never half-compiled.

use worth_query_declaration::facade::application_program::ApplicationWorkflowDataFlow;
use worth_query_installation::facade::WorthQueryWorkflowNodeDependency;
use worth_relational::facade::identity::EntityId;

use super::codec::{WorkflowConnectionTag, WorkflowNodeTag};
use crate::domain_computation::primary_graph::workflow::adoption::{
    WorkflowAdoptionReadDenial, WorkflowAdoptionTruth,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph) struct WorkflowRetainedNode {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) path: String,
    /// `None` for evidence joins and terminals, which act through no program
    /// member.
    pub(in crate::domain_computation::primary_graph) dependency:
        Option<WorthQueryWorkflowNodeDependency>,
}

pub(in crate::domain_computation::primary_graph) struct WorkflowDefinitionDependencies {
    pub(in crate::domain_computation::primary_graph) spec: String,
    pub(in crate::domain_computation::primary_graph) nodes: Vec<WorkflowRetainedNode>,
    /// `(approval, operation)` node pairs joined by approval authority.
    pub(in crate::domain_computation::primary_graph) approval_authorities:
        Vec<(EntityId, EntityId)>,
}

impl WorkflowDefinitionDependencies {
    pub(in crate::domain_computation::primary_graph) fn node(
        &self,
        entity: EntityId,
    ) -> Option<&WorkflowRetainedNode> {
        self.nodes.iter().find(|node| node.entity == entity)
    }
}

pub(in crate::domain_computation::primary_graph) fn read_definition_dependencies(
    truth: &mut WorkflowAdoptionTruth<'_>,
    layout: &WorthQueryWorkflowLayout,
    lineage: EntityId,
    definition: EntityId,
) -> Result<WorkflowDefinitionDependencies, WorkflowAdoptionReadDenial> {
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: definition };
    truth.entity(definition, layout.definition.entity_kind)?;
    let lineage_record = truth.entity(lineage, layout.lineage.entity_kind)?;
    if lineage_record.u64(&layout.lineage.protocol_version)?
        != crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION
    {
        return Err(WorkflowAdoptionReadDenial::UnreadableEntity { entity: lineage });
    }
    let spec = lineage_record.text(&layout.lineage.spec)?;
    let mut nodes = Vec::new();
    for membership in truth.outgoing(definition, layout.definition_node_relation)? {
        nodes.push(read_node(truth, layout, membership.target)?);
    }
    if nodes.is_empty() {
        return Err(unreadable);
    }
    let mut approval_authorities = Vec::new();
    for membership in truth.outgoing(definition, layout.definition_connection_relation)? {
        let connection = membership.target;
        let record = truth.entity(connection, layout.connection.entity_kind)?;
        let tag = WorkflowConnectionTag::from_persisted(
            record.u64(&layout.connection.family)?,
            record.u64(&layout.connection.variant)?,
        )
        .ok_or(WorkflowAdoptionReadDenial::UnreadableEntity { entity: connection })?;
        if !matches!(
            tag,
            WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)
        ) {
            continue;
        }
        let approval = truth.single_target(connection, layout.connection_source_relation)?;
        let operation = truth.single_target(connection, layout.connection_target_relation)?;
        approval_authorities.push((approval, operation));
    }
    let dependencies = WorkflowDefinitionDependencies {
        spec,
        nodes,
        approval_authorities,
    };
    let joined = dependencies
        .approval_authorities
        .iter()
        .all(|(approval, operation)| {
            matches!(
                dependencies
                    .node(*approval)
                    .and_then(|node| node.dependency.as_ref()),
                Some(WorthQueryWorkflowNodeDependency::Approval { .. })
            ) && matches!(
                dependencies
                    .node(*operation)
                    .and_then(|node| node.dependency.as_ref()),
                Some(WorthQueryWorkflowNodeDependency::Operation { .. })
            )
        });
    if !joined {
        return Err(unreadable);
    }
    Ok(dependencies)
}

fn read_node(
    truth: &mut WorkflowAdoptionTruth<'_>,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
) -> Result<WorkflowRetainedNode, WorkflowAdoptionReadDenial> {
    let node = &layout.node;
    let record = truth.entity(entity, node.entity_kind)?;
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity };
    let path = record.text(&node.path)?;
    let identifier = record.text(&node.member)?;
    let required = |locator: &worth_foundational::facade::AspectFieldLocator| {
        record.optional_text(locator)?.ok_or(unreadable)
    };
    let dependency = match WorkflowNodeTag::from_persisted(record.u64(&node.kind)?) {
        Some(WorkflowNodeTag::Operation) => Some(WorthQueryWorkflowNodeDependency::Operation {
            input_type: required(&node.input_type)?,
            binding: record
                .optional_text(&node.operation_binding)?
                .filter(|binding| !binding.is_empty()),
            requires_authority: record.bool(&node.requires_authority)?,
            identifier,
        }),
        Some(WorkflowNodeTag::Assessment) => Some(WorthQueryWorkflowNodeDependency::Assessment {
            parameter_type: required(&node.parameter_type)?,
            result_type: required(&node.result_type)?,
            binding: required(&node.assessment_binding)?,
            identifier,
        }),
        Some(WorkflowNodeTag::Condition) => Some(WorthQueryWorkflowNodeDependency::Condition {
            parameter_type: required(&node.parameter_type)?,
            result_type: required(&node.result_type)?,
            binding: required(&node.condition_binding)?,
            identifier,
        }),
        Some(WorkflowNodeTag::Approval) => Some(WorthQueryWorkflowNodeDependency::Approval {
            capability_type: required(&node.capability_type)?,
            operation: required(&node.approval_operation)?,
            capability_identity: required(&node.approval_capability_identity)?,
            identifier,
        }),
        Some(WorkflowNodeTag::EvidenceJoin | WorkflowNodeTag::Terminal) => None,
        None => return Err(unreadable),
    };
    Ok(WorkflowRetainedNode {
        entity,
        path,
        dependency,
    })
}
