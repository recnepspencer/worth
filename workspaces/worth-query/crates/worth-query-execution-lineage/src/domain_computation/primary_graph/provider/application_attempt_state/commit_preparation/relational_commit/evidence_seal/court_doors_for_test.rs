//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphCommitEvidence {
    #[doc(hidden)]
    pub fn rebased_source_facts_for_test(
        &self,
    ) -> Result<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        FailedRebase,
    > {
        Self::rebased_source_facts(self)
    }
}

impl WorthQueryPrimaryGraphCommitEvidence {
    #[doc(hidden)]
    pub fn original_authority_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding
    {
        Self::original_authority(self)
    }
}

impl WorthQueryPrimaryGraphCommitEvidence {
    #[doc(hidden)]
    pub fn retained_preimage_for_test(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        Self::retained_preimage(self)
    }
}
