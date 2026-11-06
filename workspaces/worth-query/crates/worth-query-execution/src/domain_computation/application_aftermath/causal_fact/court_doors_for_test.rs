//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPendingAftermathCausality {
    #[doc(hidden)]
    pub fn recorded_correction_for_test(
        correction: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >,
    ) -> Self {
        Self::recorded_correction(correction)
    }
}

impl WorthQueryPendingAftermathCausality {
    #[doc(hidden)]
    pub fn parent_for_test(&self) -> &RelationalCommitReceipt {
        Self::parent(self)
    }
}

impl WorthQueryCommittedAftermathCausality {
    #[doc(hidden)]
    pub fn owned_receipt_backing_bytes_for_test(&self) -> Option<u64> {
        Self::owned_receipt_backing_bytes(self)
    }
}
