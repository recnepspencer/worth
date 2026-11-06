//! Sealed test door over the contribution-owned admission of a producer edition: the door
//! lives with the behaviour, not on the lowered type.
use super::*;
use crate::domain_computation::primary_graph::computation_retention::producer_edition::InstalledProducerEdition;

pub trait ProducerEditionAdmissionForTest: Sized {
    #[doc(hidden)]
    fn from_declaration_for_test(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial>;
}

impl ProducerEditionAdmissionForTest for InstalledProducerEdition {
    fn from_declaration_for_test(
        declaration: &DeclaredProducerBinding,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        <Self as ProducerEditionAdmission>::from_declaration(declaration, build_identity)
    }
}
