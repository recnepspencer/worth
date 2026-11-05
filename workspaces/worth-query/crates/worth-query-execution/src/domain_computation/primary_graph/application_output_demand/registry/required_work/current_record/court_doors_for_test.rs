//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn pin_current_required_successor_for_test(
        &self,
        work: &mut SelectedRequiredWork,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::pin_current_required_successor(self, work)
    }
}
