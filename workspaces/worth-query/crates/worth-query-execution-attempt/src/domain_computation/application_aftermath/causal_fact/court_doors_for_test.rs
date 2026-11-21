//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPendingAftermathCausality {
    #[doc(hidden)]
    pub fn recorded_correction_for_test(
        correction: std::sync::Arc<dyn crate::ports::PreparedCorrectionSourcePort>,
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
