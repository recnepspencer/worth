//! Sealed test doors over this module's private members.
use super::*;

impl HistoryReconstructionCharge {
    #[doc(hidden)]
    pub fn new_for_test(
        budget: WorthQueryWorkflowHistoryReconstructionBudget,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        Self::new(budget)
    }
}
