//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn prepare_selected_execution_finish_for_test<'a>(
        &'a self, interest: &WorthQueryOutputDemandInterest) -> Result<PreparedSelectedExecutionFinish<'a>, WorthQueryOutputDemandDenial> {
        Self::prepare_selected_execution_finish(self, interest)
    }
}
