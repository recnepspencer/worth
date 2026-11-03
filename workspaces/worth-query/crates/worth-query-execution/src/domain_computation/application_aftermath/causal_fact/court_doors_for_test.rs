//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPendingAftermathCausality {
    #[doc(hidden)]
    pub fn recorded_correction_for_test( correction: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >) -> Self {
        Self::recorded_correction(correction)
    }
}
