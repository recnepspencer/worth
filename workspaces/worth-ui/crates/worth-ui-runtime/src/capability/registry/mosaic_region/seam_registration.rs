#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MosaicSeamPaintAcceptedRegistrationProof {
    accepted_identity_texts: std::collections::BTreeSet<String>,
}

impl MosaicSeamPaintAcceptedRegistrationProof {
    pub(crate) fn from_identity_texts(
        accepted_identity_texts: std::collections::BTreeSet<String>,
    ) -> Self {
        Self {
            accepted_identity_texts,
        }
    }

    pub(crate) fn admits_contract(&self) -> bool {
        self.accepted_identity_texts
            .contains(super::MosaicRegionRegistry::SEAM_REGISTRATION_IDENTITY)
    }
}
