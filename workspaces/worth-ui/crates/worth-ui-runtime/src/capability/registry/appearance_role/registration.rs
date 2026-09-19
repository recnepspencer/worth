use std::collections::BTreeSet;

pub(crate) struct AppearanceRoleAcceptedRegistrationProof {
    identities: BTreeSet<String>,
}

impl AppearanceRoleAcceptedRegistrationProof {
    pub(crate) fn from_identity_texts(identities: BTreeSet<String>) -> Self {
        Self { identities }
    }

    pub(crate) fn admits(&self, role: &worth_ui_dsl::UiAppearanceRoleDeclaration) -> bool {
        self.identities.contains(role.role().as_str())
    }
}
