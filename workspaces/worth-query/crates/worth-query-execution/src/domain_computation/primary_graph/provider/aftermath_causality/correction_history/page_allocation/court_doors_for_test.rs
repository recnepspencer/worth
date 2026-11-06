//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCorrectionHistoryPageAllocation {
    #[doc(hidden)]
    pub fn new_for_test(execution: &ExecutionRequest<'_, '_>) -> Result<Self, Denial> {
        Self::new(execution)
    }
}
