use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn empty_node_fields(
    member: &str,
    input_type: &Option<String>,
    parameter_type: &Option<String>,
    result_type: &Option<String>,
    assessment_binding: &Option<String>,
    assessment_subject: &Option<String>,
    condition_binding: &Option<String>,
    capability_type: &Option<String>,
    approval_operation: &Option<String>,
    approval_capability_identity: &Option<String>,
    requires_workflow_authority: bool,
) -> bool {
    member.is_empty()
        && empty_optional_node_fields(
            input_type,
            parameter_type,
            result_type,
            assessment_binding,
            assessment_subject,
            condition_binding,
            capability_type,
            approval_operation,
            approval_capability_identity,
            requires_workflow_authority,
        )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn empty_optional_node_fields(
    input_type: &Option<String>,
    parameter_type: &Option<String>,
    result_type: &Option<String>,
    assessment_binding: &Option<String>,
    assessment_subject: &Option<String>,
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
        && assessment_subject.is_none()
        && condition_binding.is_none()
        && capability_type.is_none()
        && approval_operation.is_none()
        && approval_capability_identity.is_none()
        && !requires_workflow_authority
}

pub(super) fn invalid_node() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable,
        "published workflow node shape is invalid",
    )
}
