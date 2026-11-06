//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryPendingDispatchOutbox;
use crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial;

impl WorthQueryCommittedDispatchOutboxResolution {
    #[doc(hidden)]
    pub fn from_commit_for_test(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Self {
        Self::from_commit(pending, committed)
    }
}

impl WorthQueryCommittedDispatchOutboxResolution {
    #[doc(hidden)]
    pub fn seal_for_receipt_for_test(
        &self,
    ) -> Result<
        WorthQueryCommittedDispatchOutboxReceiptSeal,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    > {
        Self::seal_for_receipt(self)
    }
}
