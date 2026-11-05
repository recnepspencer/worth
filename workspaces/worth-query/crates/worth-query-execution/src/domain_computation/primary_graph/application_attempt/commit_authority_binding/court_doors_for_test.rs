//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn corrupt_original_disclosure_for_test(
        &mut self,
        disclosure: Arc<super::super::super::source_disclosure::OriginalSourceInspectionDisclosure>,
    ) {
        Self::corrupt_original_disclosure(self, disclosure)
    }
}

impl WorthQueryRetainedGovernedInput {
    #[doc(hidden)]
    pub fn axis_probe_for_test(governed_identity: [u8; 32]) -> Self {
        Self::axis_probe(governed_identity)
    }
}

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn original_inspection_disclosure_for_test(
        &self,
    ) -> Option<&super::super::super::source_disclosure::OriginalSourceInspectionDisclosure> {
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
