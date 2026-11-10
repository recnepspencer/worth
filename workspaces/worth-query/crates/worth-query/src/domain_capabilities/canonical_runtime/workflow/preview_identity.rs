use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::BridgePreviewSessionDeclarationIdentity;

use crate::domain_capabilities::payloads::WorthQueryWorkflowContributionPayload;
use crate::identity::{CanonicalQueryDigest, ValidatedQueryDigest};

pub(super) fn preview_canonical_query_identity(
    source_label: &str,
    binding_identity: &crate::WorthQueryEvidenceIdentity,
    request_identity: &crate::WorthQueryEvidenceIdentity,
    preview_session_identity: &worth_runtime_bridge::facade::BridgePreviewSessionIdentity,
    evaluation_class: &crate::workflow::WorkflowPreviewEvaluationClass,
    request_family_label: &str,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "worth_query_domain_preview_query_v1",
    )
    .field_shape(crate::WorthQueryEvidenceTag::new("source"), source_label)
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("binding"),
        binding_identity,
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("request"),
        request_identity,
    )
    .field_bridge_authority_identity(
        crate::WorthQueryEvidenceTag::new("preview_session"),
        &preview_session_identity.bridge_trust_boundary(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("evaluation"),
        evaluation_class.as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("request_family"),
        request_family_label,
    )
    .seal()
}

pub(super) fn preview_validated_query_identity(
    canonical_query_identity: &crate::WorthQueryEvidenceIdentity,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "worth_query_domain_preview_validated_query_v1",
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("canonical"),
        canonical_query_identity,
    )
    .seal()
}

pub(super) fn preview_declaration_identity(
    payload: &WorthQueryWorkflowContributionPayload,
    binding_identity: &crate::WorthQueryEvidenceIdentity,
    request_identity: &crate::WorthQueryEvidenceIdentity,
    preview_session_identity: &worth_runtime_bridge::facade::BridgePreviewSessionIdentity,
    evaluation_class: &crate::workflow::WorkflowPreviewEvaluationClass,
    request_family_label: &str,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "domain_preview_declaration_v1",
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("semantic_code"),
        payload.semantic_code(),
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("binding"),
        binding_identity,
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("request"),
        request_identity,
    )
    .field_bridge_authority_identity(
        crate::WorthQueryEvidenceTag::new("preview_session"),
        &preview_session_identity.bridge_trust_boundary(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("evaluation"),
        evaluation_class.as_str(),
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("request_family"),
        request_family_label,
    )
    .seal()
}

pub(super) fn canonical_query_digest_from_identity(
    identity: &crate::WorthQueryEvidenceIdentity,
) -> CanonicalQueryDigest {
    CanonicalQueryDigest::from_canonical_digest(identity.canonical_digest())
}

pub(super) fn validated_query_digest_from_identity(
    identity: &crate::WorthQueryEvidenceIdentity,
) -> ValidatedQueryDigest {
    ValidatedQueryDigest::from_canonical_digest(identity.canonical_digest())
}

pub(super) fn sealed_preview_declaration_bridge_identity(
    identity: &crate::WorthQueryEvidenceIdentity,
) -> BridgePreviewSessionDeclarationIdentity {
    BridgePreviewSessionDeclarationIdentity::from_bridge_evidence(
        &identity.bridge_external_identity_evidence(),
    )
}

pub(super) fn preview_declaration_digest_identity(
    preview_declaration_identity: &crate::WorthQueryEvidenceIdentity,
    canonical_query_digest: &CanonicalQueryDigest,
    validated_query_digest: &ValidatedQueryDigest,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "worth_query_domain_preview_declaration_v1",
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("declaration"),
        preview_declaration_identity,
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("canonical"),
        &domain_preview_canonical_query_digest_evidence(canonical_query_digest)?,
    )
    .field_evidence_identity(
        crate::WorthQueryEvidenceTag::new("validated"),
        &domain_preview_validated_query_digest_evidence(validated_query_digest)?,
    )
    .seal()
}

fn domain_preview_canonical_query_digest_evidence(
    digest: &CanonicalQueryDigest,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "domain_preview_canonical_query_digest_evidence_v1",
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("canonical_query_digest"),
        digest.as_str(),
    )
    .seal()
}

fn domain_preview_validated_query_digest_evidence(
    digest: &ValidatedQueryDigest,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::WorkflowContextBinding,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "domain_preview_validated_query_digest_evidence_v1",
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("validated_query_digest"),
        digest.as_str(),
    )
    .seal()
}
