//! Sealed test doors over this module's private members.
use super::*;

impl WorthQuerySessionReadAuthority<'_> {
    #[doc(hidden)]
    pub fn compare_decision_read_set_with_for_test(
        &self, receipt: WorthQueryCompleteDecisionReadSetReceipt, compare: impl FnMut(
            super::super::WorthQueryDecisionFactEvidenceView<'_>,
            WorthQueryDecisionFactComparisonAdmission,
        ) -> Result<
            WorthQueryDecisionFactComparisonEvidence,
            WorthQueryDecisionReadSetFailure,
        >) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        Self::compare_decision_read_set_with(self, receipt, compare)
    }
}

impl WorthQueryFreshDecisionReadSet {
    #[doc(hidden)]
    pub fn contains_locator_for_test(&self, identity: &str) -> bool {
        Self::contains_locator(self, identity)
    }
}
