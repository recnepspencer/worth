use worth_relational::facade::identity::EntityId;

use super::{observed_bool, observed_optional_text, observed_text, observed_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{
        codec::WorkflowNodeTag,
        compilation::plan::{CompiledWorkflowNode, CompiledWorkflowNodeKind},
    },
    schema::WorthQueryWorkflowLayout,
};

pub(super) fn compile_node(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let node = &layout.node;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind: node.entity_kind,
    });
    let path = observed_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.path,
        facts,
    )?;
    let kind = observed_u64(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.kind,
        facts,
    )?;
    let member = observed_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.member,
        facts,
    )?;
    let input_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.input_type,
        facts,
    )?;
    let parameter_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.parameter_type,
        facts,
    )?;
    let result_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.result_type,
        facts,
    )?;
    let assessment_binding = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_binding,
        facts,
    )?;
    let condition_binding = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.condition_binding,
        facts,
    )?;
    let capability_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.capability_type,
        facts,
    )?;
    let approval_operation = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.approval_operation,
        facts,
    )?;
    let approval_capability_identity = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.approval_capability_identity,
        facts,
    )?;
    let requires_workflow_authority = observed_bool(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.requires_authority,
        facts,
    )?;
    let kind = decode_kind(
        kind,
        member,
        input_type,
        parameter_type,
        result_type,
        assessment_binding,
        condition_binding,
        capability_type,
        approval_operation,
        approval_capability_identity,
        requires_workflow_authority,
    )?;
    Ok(CompiledWorkflowNode { entity, path, kind })
}

#[allow(clippy::too_many_arguments)]
fn decode_kind(
    tag: u64,
    member: String,
    input_type: Option<String>,
    parameter_type: Option<String>,
    result_type: Option<String>,
    assessment_binding: Option<String>,
    condition_binding: Option<String>,
    capability_type: Option<String>,
    approval_operation: Option<String>,
    approval_capability_identity: Option<String>,
    requires_workflow_authority: bool,
) -> Result<CompiledWorkflowNodeKind, WorthQueryApplicationAttemptDenial> {
    match WorkflowNodeTag::from_persisted(tag) {
        Some(WorkflowNodeTag::Operation)
            if parameter_type.is_none()
                && result_type.is_none()
                && assessment_binding.is_none()
                && condition_binding.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none() =>
        {
            Ok(CompiledWorkflowNodeKind::Operation {
                operation: member,
                input_type: input_type.ok_or_else(invalid_node)?,
                requires_workflow_authority,
            })
        }
        Some(WorkflowNodeTag::Assessment)
            if input_type.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none()
                && condition_binding.is_none()
                && !requires_workflow_authority =>
        {
            Ok(CompiledWorkflowNodeKind::Assessment {
                query: member,
                parameter_type: parameter_type.ok_or_else(invalid_node)?,
                result_type: result_type.ok_or_else(invalid_node)?,
                binding: assessment_binding.ok_or_else(invalid_node)?,
            })
        }
        Some(WorkflowNodeTag::Condition)
            if input_type.is_none()
                && assessment_binding.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none()
                && !requires_workflow_authority =>
        {
            Ok(CompiledWorkflowNodeKind::Condition {
                query: member,
                parameter_type: parameter_type.ok_or_else(invalid_node)?,
                result_type: result_type.ok_or_else(invalid_node)?,
                binding: condition_binding.ok_or_else(invalid_node)?,
            })
        }
        Some(WorkflowNodeTag::Approval)
            if input_type.is_none()
                && parameter_type.is_none()
                && result_type.is_none()
                && assessment_binding.is_none()
                && condition_binding.is_none()
                && !requires_workflow_authority =>
        {
            Ok(CompiledWorkflowNodeKind::Approval {
                capability: member,
                capability_type: capability_type.ok_or_else(invalid_node)?,
                operation: approval_operation.ok_or_else(invalid_node)?,
                installed_capability_identity: approval_capability_identity
                    .ok_or_else(invalid_node)?,
            })
        }
        Some(WorkflowNodeTag::EvidenceJoin)
            if empty_optional_node_fields(
                &input_type,
                &parameter_type,
                &result_type,
                &assessment_binding,
                &condition_binding,
                &capability_type,
                &approval_operation,
                &approval_capability_identity,
                requires_workflow_authority,
            ) =>
        {
            let policy = worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy::from_identity(&member)
                .ok_or_else(invalid_node)?;
            Ok(CompiledWorkflowNodeKind::EvidenceJoin { policy })
        }
        Some(WorkflowNodeTag::Terminal)
            if empty_node_fields(
                &member,
                &input_type,
                &parameter_type,
                &result_type,
                &assessment_binding,
                &condition_binding,
                &capability_type,
                &approval_operation,
                &approval_capability_identity,
                requires_workflow_authority,
            ) =>
        {
            Ok(CompiledWorkflowNodeKind::Terminal)
        }
        _ => Err(invalid_node()),
    }
}

#[allow(clippy::too_many_arguments)]
fn empty_node_fields(
    member: &str,
    input_type: &Option<String>,
    parameter_type: &Option<String>,
    result_type: &Option<String>,
    assessment_binding: &Option<String>,
    condition_binding: &Option<String>,
    capability_type: &Option<String>,
    approval_operation: &Option<String>,
    approval_capability_identity: &Option<String>,
    requires_workflow_authority: bool,
) -> bool {
    member.is_empty()
        && input_type.is_none()
        && parameter_type.is_none()
        && result_type.is_none()
        && assessment_binding.is_none()
        && condition_binding.is_none()
        && capability_type.is_none()
        && approval_operation.is_none()
        && approval_capability_identity.is_none()
        && !requires_workflow_authority
}

#[allow(clippy::too_many_arguments)]
fn empty_optional_node_fields(
    input_type: &Option<String>,
    parameter_type: &Option<String>,
    result_type: &Option<String>,
    assessment_binding: &Option<String>,
    condition_binding: &Option<String>,
    capability_type: &Option<String>,
    approval_operation: &Option<String>,
    approval_capability_identity: &Option<String>,
    requires_workflow_authority: bool,
) -> bool {
    input_type.is_none()
        && parameter_type.is_none()
        && result_type.is_none()
        && assessment_binding.is_none()
        && condition_binding.is_none()
        && capability_type.is_none()
        && approval_operation.is_none()
        && approval_capability_identity.is_none()
        && !requires_workflow_authority
}

fn invalid_node() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable,
        "published workflow node shape is invalid",
    )
}
