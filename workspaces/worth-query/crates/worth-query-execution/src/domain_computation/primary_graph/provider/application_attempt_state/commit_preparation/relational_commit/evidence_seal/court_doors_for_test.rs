//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphCommitEvidence {
    #[doc(hidden)]
    pub fn rebased_source_facts_for_test(
        &self) -> Result<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        FailedRebase,
    > {
        Self::rebased_source_facts(self)
    }
}
