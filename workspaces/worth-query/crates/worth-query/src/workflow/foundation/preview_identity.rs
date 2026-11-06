use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::preview::{
    preview_lifecycle_state_label, AdmittedPreviewWorkflowFoundation,
    PromotionParityPreviewComparisonAdmission,
};

use super::context_identity::{
    workflow_canonical_query_digest_evidence, workflow_validated_query_digest_evidence,
};

pub(super) fn preview_workflow_foundation_binding_identity(
    foundation: &AdmittedPreviewWorkflowFoundation,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let artifact = foundation.artifact();
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_workflow_foundation_binding_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("request"),
                foundation.request_family().as_str(),
            )
            .field_bridge_authority_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &foundation
                    .preview_session_identity()
                    .bridge_trust_boundary(),
            )
            .field_bridge_authority_identity(
                WorthQueryEvidenceTag::new("declaration"),
                &foundation.declaration_identity().bridge_trust_boundary(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(foundation.validated_query_digest())?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical_query"),
                &workflow_canonical_query_digest_evidence(foundation.canonical_query_digest())?,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                preview_lifecycle_state_label(artifact.lifecycle_state_kind()),
            )
            .field_bridge_authority_identity(
                WorthQueryEvidenceTag::new("execution_record"),
                &foundation
                    .execution_record_identity()
                    .bridge_trust_boundary(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation_class"),
                artifact.evaluation_class().as_str(),
            )
            .seal()?,
    )
}

pub(super) fn preview_workflow_foundation_source_identity(
    foundation: &AdmittedPreviewWorkflowFoundation,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_workflow_foundation_source_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &preview_workflow_foundation_binding_identity(foundation)?,
            )
            .seal()?,
    )
}

pub(super) fn preview_workflow_foundation_basis_inner_identity(
    foundation: &AdmittedPreviewWorkflowFoundation,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_workflow_foundation_basis_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &preview_workflow_foundation_binding_identity(foundation)?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(foundation.validated_query_digest())?,
            )
            .field_bridge_authority_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &foundation
                    .preview_session_identity()
                    .bridge_trust_boundary(),
            )
            .seal()?,
    )
}

pub(super) fn preview_promotion_comparison_binding_identity(
    comparison: &PromotionParityPreviewComparisonAdmission,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_promotion_comparison_binding_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(comparison.validated_query_digest())?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical_query"),
                &workflow_canonical_query_digest_evidence(comparison.canonical_query_digest())?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("candidate_result"),
                &preview_candidate_result_identity(comparison.candidate_result_digest())?,
            )
            .seal()?,
    )
}

fn preview_candidate_result_identity(
    result_digest: &crate::identity::ResultDigest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_candidate_result_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("result_label"),
                result_digest.as_str(),
            )
            .seal()?,
    )
}

pub(super) fn preview_promotion_comparison_source_identity(
    comparison: &PromotionParityPreviewComparisonAdmission,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_promotion_comparison_source_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &preview_promotion_comparison_binding_identity(comparison)?,
            )
            .seal()?,
    )
}

pub(super) fn preview_promotion_comparison_basis_inner_identity(
    comparison: &PromotionParityPreviewComparisonAdmission,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_promotion_comparison_basis_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &preview_promotion_comparison_binding_identity(comparison)?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(comparison.validated_query_digest())?,
            )
            .seal()?,
    )
}
