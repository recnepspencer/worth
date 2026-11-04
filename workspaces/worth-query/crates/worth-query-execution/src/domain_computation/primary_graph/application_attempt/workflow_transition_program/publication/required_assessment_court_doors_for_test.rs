//! Sealed test doors over this module's private members.
use super::*;

impl RequiredWorkflowAssessment {
    #[doc(hidden)]
    pub fn coverage_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.coverage_identity
    }
}

impl RequiredWorkflowAssessment {
    #[doc(hidden)]
    pub fn proposal_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.proposal_identity
    }
}
