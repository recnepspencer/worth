//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn prepare_selected_checkpoint_finish_for_test<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Result<PreparedSelectedCheckpointFinish<'a>, WorthQueryOutputDemandDenial> {
        Self::prepare_selected_checkpoint_finish(self, interest)
    }
}
