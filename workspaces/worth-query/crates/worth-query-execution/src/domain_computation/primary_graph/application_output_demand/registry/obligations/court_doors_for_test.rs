//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn release_record_prerequisites_for_test(
        &mut self, key: &WorthQueryOutputDemandKey) -> Vec<Arc<WorthQueryOutputDemandKey>> {
        Self::release_record_prerequisites(self, key)
    }
}
