use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::BoundaryArtifactId;

use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::domain_capabilities::payloads::WorthQueryDomainCapabilityCategory;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetKind;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(crate) fn diagnostic_scope_identity(
    category: WorthQueryDomainCapabilityCategory,
    target_kind: WorthQueryDomainCapabilityTargetKind,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_diagnostic_scope_v1")
            .field_shape(WorthQueryEvidenceTag::new("category"), category.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("target_kind"),
                target_kind.as_str(),
            )
            .seal()?,
    )
}

pub(crate) fn diagnostic_label_identity(
    role: &'static str,
    value: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_diagnostic_label_v1")
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(WorthQueryEvidenceTag::new("value"), value)
            .seal()?,
    )
}

pub(crate) fn diagnostic_code_identity(
    role: &'static str,
    semantic_code: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_diagnostic_code_v1")
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
            .seal()?,
    )
}

pub(crate) fn boundary_artifact_id(identity: &WorthQueryEvidenceIdentity) -> BoundaryArtifactId {
    crate::domain_installation::foundational_boundary_artifact_id(identity)
}
