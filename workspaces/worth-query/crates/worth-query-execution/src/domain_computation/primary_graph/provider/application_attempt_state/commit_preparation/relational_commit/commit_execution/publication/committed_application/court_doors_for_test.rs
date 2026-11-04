//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphCommittedApplication {
    #[doc(hidden)]
    pub fn commit_reference_for_test(&self) -> &RelationalCommitReceipt {
        Self::commit_reference(self)
    }
}
