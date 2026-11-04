//! Sealed test doors over this module's private members.
use super::*;

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn for_test_for_test(digest: [u8; 32]) -> Self {
        Self::for_test(digest)
    }
}

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn build_identity_for_test(self) -> worth_foundational::facade::ApplicationBuildIdentity {
        Self::build_identity(self)
    }
}

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn digest_for_test(self) -> [u8; 32] {
        Self::digest(self)
    }
}

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn from_declaration_for_test(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        Self::from_declaration(declaration, build_identity)
    }
}
