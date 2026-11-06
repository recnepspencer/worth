//! Sealed test doors over this module's private members.
use super::*;

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn from_declaration_for_test(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        <Self as ProducerEditionAdmission>::from_declaration(declaration, build_identity)
    }
}
