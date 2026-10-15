//! Existing durable approval fixture transport under its real host execution.
use super::*;

pub(super) fn durable_approval_basis(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &Admission,
) -> WorthQueryWorkflowApprovalAuthorityBasis {
    durable_approval_basis_with_snapshots(execution, admission, None, None)
}

pub(super) fn durable_approval_basis_with_snapshots(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &Admission,
    replacement_support: Option<&str>,
    replacement_lineage: Option<&str>,
) -> WorthQueryWorkflowApprovalAuthorityBasis {
    durable_approval_basis_with_dependency_snapshot(
        execution,
        admission,
        replacement_support,
        replacement_lineage,
        None,
    )
}

pub(super) fn durable_approval_basis_with_dependency_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: &Admission,
    replacement_support: Option<&str>,
    replacement_lineage: Option<&str>,
    replacement_dependencies: Option<&str>,
) -> WorthQueryWorkflowApprovalAuthorityBasis {
    let authority = admission.workflow_approval_authority().unwrap();
    let request = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
        &execution,
        &admission
            .workflow_approval_request_snapshot(&execution)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let principal = crate::domain_computation::authorization::workflow_approval_authority::decoding::decode_principal(
        &execution,
        &admission
            .workflow_approval_principal_snapshot(&execution)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let original_lineage = admission
        .workflow_approval_lineage_snapshot(&execution)
        .unwrap()
        .unwrap();
    let lineage = crate::domain_computation::authorization::workflow_approval_authority::decoding::decode_lineage(
        &execution, replacement_lineage.unwrap_or(&original_lineage)).unwrap();
    let original_support = admission
        .workflow_approval_support_snapshot(&execution)
        .unwrap()
        .unwrap();
    let support = decode_workflow_approval_support(
        &execution,
        replacement_support.unwrap_or(&original_support),
    )
    .unwrap();
    let original_dependencies = admission
        .workflow_approval_dependencies_snapshot(&execution)
        .unwrap()
        .unwrap();
    let mut dependency_snapshot: serde_json::Value =
        serde_json::from_str(replacement_dependencies.unwrap_or(&original_dependencies)).unwrap();
    if support.is_some() && dependency_snapshot["support"].is_null() {
        dependency_snapshot["support"] = dependency_snapshot["primary"].clone();
    }
    let dependencies =
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalDependencies::decode(
            &execution,
            &dependency_snapshot.to_string(),
        )
        .unwrap();
    let worth_foundational::facade::AspectValue::UInt64(expiry) = authority.expiry else {
        panic!("the fixture uses an unsigned capability expiry");
    };
    WorthQueryWorkflowApprovalAuthorityBasis::from_retained_approval(
        request,
        principal,
        authority.grant,
        authority.capability_authority_identity,
        lineage,
        support,
        dependencies,
        authority.timeline,
        expiry,
    )
}
