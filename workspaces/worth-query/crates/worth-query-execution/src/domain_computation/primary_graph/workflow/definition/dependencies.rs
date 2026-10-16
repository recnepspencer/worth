//! What one published definition depends on, read from its retained facts.
//!
//! Program adoption compares these facts with installed vocabulary coverage.
//! It never reconstructs a compiled plan, so a definition the target cannot
//! execute is still inventoried exactly and never half-compiled.

use worth_query_declaration::facade::application_program::ApplicationWorkflowDataFlow;
use worth_query_installation::facade::WorthQueryWorkflowNodeDependency;
use worth_relational::facade::identity::EntityId;

use super::codec::{WorkflowConnectionTag, WorkflowNodeTag};
use super::CompiledWorkflowCondition;
use crate::domain_computation::primary_graph::workflow::adoption::{
    WorkflowAdoptionReadDenial, WorkflowAdoptionTruth,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph) struct WorkflowRetainedNode {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) path: String,
    pub(in crate::domain_computation::primary_graph) inbound_origin: Option<String>,
    /// Empty for evidence joins and terminals, which act through no program
    /// member; one per operand for a condition; otherwise one.
    pub(in crate::domain_computation::primary_graph) dependencies:
        Vec<WorthQueryWorkflowNodeDependency>,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    truth: &mut WorkflowAdoptionTruth,
    layout: &WorthQueryWorkflowLayout,
    lineage: EntityId,
    definition: EntityId,
) -> Result<WorkflowDefinitionDependencies, WorkflowAdoptionReadDenial> {
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: definition };
    truth.entity(execution, definition, layout.definition.entity_kind)?;
    let lineage_record = truth.entity(execution, lineage, layout.lineage.entity_kind)?;
    if lineage_record.u64(execution, &layout.lineage.protocol_version)?
        != crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION
    {
        return Err(WorkflowAdoptionReadDenial::UnreadableEntity { entity: lineage });
    }
    let spec = lineage_record.text(execution, &layout.lineage.spec)?;
    let mut nodes = Vec::new();
    for membership in truth.outgoing(execution, definition, layout.definition_node_relation)? {
        nodes.push(read_node(execution, truth, layout, membership.target)?);
    }
    if nodes.is_empty() {
        return Err(unreadable);
    }
    let mut operation_by_path = std::collections::BTreeMap::new();
    for node in &nodes {
        if let [WorthQueryWorkflowNodeDependency::Operation { identifier, .. }] =
            node.dependencies.as_slice()
            && operation_by_path
                .insert(node.path.clone(), identifier.clone())
                .is_some()
        {
            return Err(unreadable);
        }
    }
    for node in &mut nodes {
        if let Some(origin) = &node.inbound_origin {
            let operation = operation_by_path.get(origin).ok_or(unreadable)?;
            let [WorthQueryWorkflowNodeDependency::AwaitInbound {
                origin_operation, ..
            }] = node.dependencies.as_mut_slice()
            else {
                return Err(unreadable);
            };
            *origin_operation = operation.clone();
        }
    }
    let mut approval_authorities = Vec::new();
    for membership in
        truth.outgoing(execution, definition, layout.definition_connection_relation)?
    {
        let connection = membership.target;
        let record = truth.entity(execution, connection, layout.connection.entity_kind)?;
        let tag = WorkflowConnectionTag::from_persisted(
            record.u64(execution, &layout.connection.family)?,
            record.u64(execution, &layout.connection.variant)?,
        )
        .ok_or(WorkflowAdoptionReadDenial::UnreadableEntity { entity: connection })?;
        if !matches!(
            tag,
            WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)
        ) {
            continue;
        }
        let approval =
            truth.single_target(execution, connection, layout.connection_source_relation)?;
        let operation =
            truth.single_target(execution, connection, layout.connection_target_relation)?;
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
                    .map(|node| node.dependencies.as_slice()),
                Some([WorthQueryWorkflowNodeDependency::Approval { .. }])
            ) && matches!(
                dependencies
                    .node(*operation)
                    .map(|node| node.dependencies.as_slice()),
                Some([WorthQueryWorkflowNodeDependency::Operation { .. }])
            )
        });
    if !joined {
        return Err(unreadable);
    }
    Ok(dependencies)
}

fn read_node(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    truth: &mut WorkflowAdoptionTruth,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
) -> Result<WorkflowRetainedNode, WorkflowAdoptionReadDenial> {
    let node = &layout.node;
    let record = truth.entity(execution, entity, node.entity_kind)?;
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity };
    let path = record.text(execution, &node.path)?;
    let identifier = record.text(execution, &node.member)?;
    let inbound_origin = record.optional_text(execution, &node.inbound_origin)?;
    let required = |locator: &worth_foundational::facade::AspectFieldLocator| {
        record.optional_text(execution, locator)?.ok_or(unreadable)
    };
    let dependencies = match WorkflowNodeTag::from_persisted(record.u64(execution, &node.kind)?) {
        Some(WorkflowNodeTag::Operation) => {
            vec![WorthQueryWorkflowNodeDependency::Operation {
                input_type: required(&node.input_type)?,
                binding: record
                    .optional_text(execution, &node.operation_binding)?
                    .filter(|binding| !binding.is_empty()),
                requires_authority: record.bool(execution, &node.requires_authority)?,
                workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements::from_persistence_identity(
                    &required(&node.workflow_execution_requirements)?).ok_or(unreadable)?,
                identifier,
            }]
        }
        Some(WorkflowNodeTag::AwaitInbound) => {
            let origin = inbound_origin.as_deref().ok_or(unreadable)?;
            if origin.is_empty()
                || record.u64(execution, &node.inbound_wait)? != 0
                || identifier.is_empty()
            {
                return Err(unreadable);
            }
            let contract = required(&node.inbound_contract)?;
            let (protocol, source_identity, limits) =
                super::inbound_codec::decode(&contract).ok_or(unreadable)?;
            vec![WorthQueryWorkflowNodeDependency::AwaitInbound {
                origin_operation: String::new(),
                effect: identifier,
                protocol,
                source_identity,
                limits,
            }]
        }
        Some(WorkflowNodeTag::Assessment) => vec![WorthQueryWorkflowNodeDependency::Assessment {
            parameter_type: required(&node.parameter_type)?,
            result_type: required(&node.result_type)?,
            binding: required(&node.assessment_binding)?,
            identifier,
        }],
        Some(WorkflowNodeTag::Condition) => CompiledWorkflowCondition::from_record(
            identifier,
            record.optional_text(execution, &node.parameter_type)?,
            record.optional_text(execution, &node.result_type)?,
            record.optional_text(execution, &node.condition_binding)?,
            record.optional_text(execution, &node.condition_operands)?,
        )
        .ok_or(unreadable)?
        .operands
        .into_vec()
        .into_iter()
        .map(|operand| WorthQueryWorkflowNodeDependency::Condition {
            identifier: operand.query,
            parameter_type: operand.parameter_type,
            result_type: operand.result_type,
            binding: operand.binding,
        })
        .collect(),
        Some(WorkflowNodeTag::Approval) => vec![WorthQueryWorkflowNodeDependency::Approval {
            capability_type: required(&node.capability_type)?,
            operation: required(&node.approval_operation)?,
            capability_identity: required(&node.approval_capability_identity)?,
            identifier,
        }],
        Some(WorkflowNodeTag::EvidenceJoin | WorkflowNodeTag::Terminal) => Vec::new(),
        None => return Err(unreadable),
    };
    Ok(WorkflowRetainedNode {
        entity,
        path,
        inbound_origin,
        dependencies,
    })
}
