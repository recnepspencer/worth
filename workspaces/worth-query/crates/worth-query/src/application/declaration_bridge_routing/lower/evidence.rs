use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

use crate::application::WorthQueryDeclarationEnvelope;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

pub(super) fn runtime_surface_identity<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::DeclarationBridgeLoweringIdentity)
        .field_shape(WorthQueryEvidenceTag::new("role"), "runtime-surface")
        .field_shape(
            WorthQueryEvidenceTag::new("declaration_family"),
            envelope.declaration_family_key(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("handle"),
            envelope.handle_identity_digest(),
        )
        .seal()
}

pub(super) fn bridge_lowering_evidence_identity(
    role: &'static str,
    evidence: impl AsRef<str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::DeclarationBridgeLoweringIdentity)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value(WorthQueryEvidenceTag::new("evidence"), evidence)
        .seal()
}

pub(super) fn bridge_lowering_bridge_evidence_identity(
    role: &'static str,
    evidence: impl AsRef<str>,
) -> Result<BridgeIdentityEvidence, IdentityDerivationDenial> {
    Ok(bridge_lowering_evidence_identity(role, evidence)?.bridge_external_identity_evidence())
}
