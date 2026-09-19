#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppearanceThemeAcceptedRegistrationProof {
    accepted_identity_texts: std::collections::BTreeSet<String>,
}

impl AppearanceThemeAcceptedRegistrationProof {
    pub(crate) fn from_identity_texts(
        accepted_identity_texts: std::collections::BTreeSet<String>,
    ) -> Self {
        Self {
            accepted_identity_texts,
        }
    }

    pub(crate) fn admits_bundle(&self) -> bool {
        self.accepted_identity_texts
            .contains(super::ThemeRegistry::REGISTRATION_IDENTITY)
    }
}
