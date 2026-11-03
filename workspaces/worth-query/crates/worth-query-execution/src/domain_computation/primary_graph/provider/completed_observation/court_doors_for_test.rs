//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_completed_application_for_test(
        &self, commit: &worth_relational::facade::history::RelationalCommitReceipt) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        Self::observe_completed_application(self, commit)
    }
}
