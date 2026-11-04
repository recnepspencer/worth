//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationHistoricalRead {
    #[doc(hidden)]
    pub fn at_application_commit_for_test(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        Self::at_application_commit(receipt)
    }
}
