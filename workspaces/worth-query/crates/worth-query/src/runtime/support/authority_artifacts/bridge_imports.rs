use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryMutationAuthorityIdentity, WorthQueryMutationSymbolIdentity};
use worth_runtime_bridge::facade::{
    bridge_truth_external_identity_token, BridgeContinuityAuthoritativeIdentity,
    BridgeExistingTruthBindingAuthoritativeIdentity, BridgeIdentityEvidence,
    BridgeNamingAttachmentIdentity, BridgeNamingAuthoritativeIdentity,
    BridgeSymbolicTargetSymbolIdentity,
};

impl WorthQueryMutationAuthorityIdentity {
    pub(in crate::runtime) fn from_bridge_existing_truth_authority(
        role: &'static str,
        identity: &BridgeExistingTruthBindingAuthoritativeIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::from_bridge_authority(
            role,
            "existing-truth",
            identity.as_str(),
        )?)
    }

    pub(in crate::runtime) fn from_bridge_naming_attachment(
        role: &'static str,
        identity: &BridgeNamingAttachmentIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::from_bridge_authority(
            role,
            "naming-attachment",
            identity.as_str(),
        )?)
    }

    pub(in crate::runtime) fn from_bridge_naming_authority(
        role: &'static str,
        identity: &BridgeNamingAuthoritativeIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::from_bridge_authority(
            role,
            "naming-authority",
            identity.as_str(),
        )?)
    }

    pub(in crate::runtime) fn from_bridge_continuity_authority(
        role: &'static str,
        identity: &BridgeContinuityAuthoritativeIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::from_bridge_authority(
            role,
            "continuity-authority",
            identity.as_str(),
        )?)
    }

    fn from_bridge_authority(
        role: &'static str,
        bridge_family: &'static str,
        authority_label: &str,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let bridge_evidence = bridge_evidence_from_authority_label(authority_label);
        Ok(Self {
            label: authority_label.to_string(),
            identity: bridge_import_identity(role, bridge_family, &bridge_evidence)?,
        })
    }
}

impl WorthQueryMutationSymbolIdentity {
    pub(in crate::runtime) fn from_bridge_symbolic_target(
        role: &'static str,
        identity: &BridgeSymbolicTargetSymbolIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let bridge_evidence = bridge_evidence_from_authority_label(identity.as_str());
        Ok(Self {
            label: identity.as_str().to_string(),
            identity: bridge_import_identity(role, "symbolic-target-symbol", &bridge_evidence)?,
        })
    }
}

fn bridge_evidence_from_authority_label(label: &str) -> BridgeIdentityEvidence {
    BridgeIdentityEvidence::from_external_authority(bridge_truth_external_identity_token(label))
}

fn bridge_import_identity(
    role: &'static str,
    bridge_family: &'static str,
    identity: &BridgeIdentityEvidence,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAuthorityIdentity)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(WorthQueryEvidenceTag::new("bridge_family"), bridge_family)
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_identity"),
                identity,
            )
            .seal()?,
    )
}
