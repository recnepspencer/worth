use super::*;

#[derive(Default)]
pub(super) struct InboundFields {
    pub(super) origin: Option<String>,
    pub(super) contract: Option<String>,
    pub(super) wait: Option<u64>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn decode_kind(
    tag: u64,
    member: String,
    input_type: Option<String>,
    operation_binding: Option<String>,
    inbound: InboundFields,
    parameter_type: Option<String>,
    result_type: Option<String>,
    assessment_binding: Option<String>,
    assessment_subject: Option<String>,
    assessment_applicability_relation: Option<String>,
    assessment_applicability_from: Option<String>,
    assessment_applicability_to: Option<String>,
    condition_binding: Option<String>,
    condition_operands: Option<String>,
    capability_type: Option<String>,
    approval_operation: Option<String>,
    approval_capability_identity: Option<String>,
    requires_workflow_authority: bool,
) -> Result<CompiledWorkflowNodeKind, WorthQueryApplicationAttemptDenial> {
    if matches!(
        WorkflowNodeTag::from_persisted(tag),
        Some(WorkflowNodeTag::AwaitInbound)
    ) {
        if member.is_empty()
            || input_type.is_some()
            || operation_binding.is_some()
            || parameter_type.is_some()
            || result_type.is_some()
            || assessment_binding.is_some()
            || assessment_subject.is_some()
            || assessment_applicability_relation.is_some()
            || assessment_applicability_from.is_some()
            || assessment_applicability_to.is_some()
            || condition_binding.is_some()
            || condition_operands.is_some()
            || capability_type.is_some()
            || approval_operation.is_some()
            || approval_capability_identity.is_some()
            || requires_workflow_authority
        {
            return Err(invalid_node());
        }
        let origin = inbound
            .origin
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_node)?;
        let contract = inbound.contract.ok_or_else(invalid_node)?;
        let (protocol, source_identity, limits) =
            super::super::super::super::inbound_codec::decode(&contract)
                .ok_or_else(invalid_node)?;
        if inbound.wait != Some(0) {
            return Err(invalid_node());
        }
        return Ok(CompiledWorkflowNodeKind::AwaitInbound {
            origin, effect: member, protocol, source_identity, limits,
            wait: worth_query_declaration::facade::application_program::ApplicationWorkflowInboundWait::UntilInstanceDeadline,
        });
    }
    if inbound.origin.is_some() || inbound.contract.is_some() || inbound.wait.is_some() {
        return Err(invalid_node());
    }
    if (!matches!(
        WorkflowNodeTag::from_persisted(tag),
        Some(WorkflowNodeTag::Operation)
    ) && operation_binding.is_some())
        || (requires_workflow_authority && operation_binding.as_deref().is_none_or(str::is_empty))
    {
        return Err(invalid_node());
    }
    let applicability = match (
        assessment_applicability_relation,
        assessment_applicability_from,
        assessment_applicability_to,
    ) {
        (None, None, None) => CompiledWorkflowAssessmentApplicability::Always,
        (Some(relation), Some(from), Some(to))
            if !relation.is_empty() && !from.is_empty() && !to.is_empty() =>
        {
            CompiledWorkflowAssessmentApplicability::WhenRelatedRelationPresent {
                relation,
                from,
                to,
            }
        }
        _ => return Err(invalid_node()),
    };
    if !matches!(
        WorkflowNodeTag::from_persisted(tag),
        Some(WorkflowNodeTag::Assessment)
    ) && !matches!(
        applicability,
        CompiledWorkflowAssessmentApplicability::Always
    ) {
        return Err(invalid_node());
    }
    match WorkflowNodeTag::from_persisted(tag) {
        Some(WorkflowNodeTag::Operation)
            if parameter_type.is_none()
                && result_type.is_none()
                && assessment_binding.is_none()
                && assessment_subject.is_none()
                && condition_binding.is_none()
                && condition_operands.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none() =>
        {
            Ok(CompiledWorkflowNodeKind::Operation {
                operation: member,
                input_type: input_type.ok_or_else(invalid_node)?,
                binding: operation_binding.filter(|identity| !identity.is_empty()),
                requires_workflow_authority,
            })
        }
        Some(WorkflowNodeTag::Assessment)
            if input_type.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none()
                && condition_binding.is_none()
                && condition_operands.is_none()
                && !requires_workflow_authority =>
        {
            let subject = worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector::from_persistence_identity(
                &assessment_subject.ok_or_else(invalid_node)?,
            )
            .ok_or_else(invalid_node)?;
            if !matches!(applicability, CompiledWorkflowAssessmentApplicability::Always)
                && !matches!(subject, worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector::Related)
            {
                return Err(invalid_node());
            }
            Ok(CompiledWorkflowNodeKind::Assessment {
                query: member,
                parameter_type: parameter_type.ok_or_else(invalid_node)?,
                result_type: result_type.ok_or_else(invalid_node)?,
                binding: assessment_binding.ok_or_else(invalid_node)?,
                subject,
                applicability,
            })
        }
        Some(WorkflowNodeTag::Condition)
            if input_type.is_none()
                && assessment_binding.is_none()
                && assessment_subject.is_none()
                && capability_type.is_none()
                && approval_operation.is_none()
                && approval_capability_identity.is_none()
                && !requires_workflow_authority =>
        {
            CompiledWorkflowCondition::from_record(
                member,
                parameter_type,
                result_type,
                condition_binding,
                condition_operands,
            )
            .map(CompiledWorkflowNodeKind::Condition)
            .ok_or_else(invalid_node)
        }
        Some(WorkflowNodeTag::Approval)
            if input_type.is_none()
                && parameter_type.is_none()
                && result_type.is_none()
                && assessment_binding.is_none()
                && assessment_subject.is_none()
                && condition_binding.is_none()
                && condition_operands.is_none()
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
                &assessment_subject,
                &condition_binding,
                &condition_operands,
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
                &assessment_subject,
                &condition_binding,
                &condition_operands,
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
