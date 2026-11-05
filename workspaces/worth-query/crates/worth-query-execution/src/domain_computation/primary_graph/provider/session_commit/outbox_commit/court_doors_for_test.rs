//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn fixture_from_commit_for_test(
        layout: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout,
        record: Option<&WorthQueryDispatchOutboxRecord>,
        committed: &CommitResult,
    ) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        Self::fixture_from_commit(layout, record, committed)
    }
}

impl WorthQueryCommittedDispatchOutboxResolution {
    #[doc(hidden)]
    pub fn from_commit_for_test(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Self {
        Self::from_commit(pending, committed)
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn record_for_test(&self) -> &WorthQueryDispatchOutboxRecord {
        Self::record(self)
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn record_ref_for_test(&self) -> &RecordRef {
        Self::record_ref(self)
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
