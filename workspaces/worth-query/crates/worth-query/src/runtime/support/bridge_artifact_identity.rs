use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBridgeMutationArtifactIdentity {
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBridgeMutationArtifactIdentity {
    pub(crate) fn imported(role: &'static str, artifact: impl Into<String>) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let artifact = artifact.into();
        let identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceSourceDigest)
                .field_shape(WorthQueryEvidenceTag::new("role"), role)
                .field_value(
                    WorthQueryEvidenceTag::new("imported_artifact"),
                    artifact.as_str(),
                )
                .seal()?;
        Ok(Self { identity })
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}
