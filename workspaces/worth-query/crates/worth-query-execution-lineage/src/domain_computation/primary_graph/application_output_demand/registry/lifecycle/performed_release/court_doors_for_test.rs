//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn release_unheld_performed_source_for_test(
        &mut self,
    ) -> Result<Option<ReleasedPerformedSource>, WorthQueryOutputDemandDenial> {
        Self::release_unheld_performed_source(self)
    }
}
