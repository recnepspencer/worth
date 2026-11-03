//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitAuthorityBinding {
    #[doc(hidden)]
    pub fn corrupt_original_disclosure_for_test(
        &mut self, disclosure: Arc<super::super::super::source_disclosure::OriginalSourceInspectionDisclosure>) {
        Self::corrupt_original_disclosure(self, disclosure)
    }
}

impl WorthQueryRetainedGovernedInput {
    #[doc(hidden)]
    pub fn axis_probe_for_test(governed_identity: [u8; 32]) -> Self {
        Self::axis_probe(governed_identity)
    }
}
