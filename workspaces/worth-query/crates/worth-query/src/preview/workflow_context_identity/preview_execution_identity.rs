use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};
use worth_runtime_bridge::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

use super::preview_lifecycle_identity::preview_lifecycle_state_label;

pub(in crate::preview) fn compose_preview_execution_comparison_admission_digest(
    preview_execution_digest: &str,
    preview_comparison_digest: &str,
    candidate_comparison_digest: &str,
    candidate_basis_digest: &str,
    candidate_result_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_execution_comparison_admission_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("preview_execution"),
                preview_execution_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("preview_comparison"),
                preview_comparison_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("candidate_comparison"),
                candidate_comparison_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("candidate_basis"),
                candidate_basis_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("candidate_result"),
                candidate_result_digest,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::preview) fn compose_preview_execution_report_digest(
    binding_digest: &str,
    basis_digest: &str,
    preview_session_identity: &BridgePreviewSessionIdentity,
    lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    execution_record_identity: &PreviewExecutionRecordIdentity,
    result_digest: &str,
    comparison_eligibility_digest: &str,
    workflow_foundation_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_execution_report_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("binding"), binding_digest)
            .field_shape(WorthQueryEvidenceTag::new("basis"), basis_digest)
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &preview_session_identity.bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                preview_lifecycle_state_label(lifecycle_state_kind),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("execution_record"),
                &execution_record_identity.bridge_admission_evidence(),
            )
            .field_shape(WorthQueryEvidenceTag::new("result"), result_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("comparison"),
                comparison_eligibility_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("workflow"),
                workflow_foundation_digest,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}
