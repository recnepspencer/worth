//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::commit_authority_binding::WorthQueryRetainedGovernedInput;

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn corrupt_original_disclosure_for_test(
        &mut self,
        disclosure: Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>,
    ) {
        Self::corrupt_original_disclosure(self, disclosure)
    }
}


impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn original_inspection_disclosure_for_test(
        &self,
    ) -> Option<&crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure> {
        Self::original_inspection_disclosure(self)
    }
}

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn retained_governed_input_for_test<Input: 'static>(&self) -> Option<&Input> {
        Self::retained_governed_input::<Input>(self)
    }
}

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn retained_governed_input_carrier_for_test(
        &self,
    ) -> Option<&WorthQueryRetainedGovernedInput> {
        Self::retained_governed_input_carrier(self)
    }
}

impl WorthQueryRetainedGovernedInput {
    #[doc(hidden)]
    pub fn governed_identity_for_test(&self) -> Option<[u8; 32]> {
        Self::governed_identity(self)
    }
}
