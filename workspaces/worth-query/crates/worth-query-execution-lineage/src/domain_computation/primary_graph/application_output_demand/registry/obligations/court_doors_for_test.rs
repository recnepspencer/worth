//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn release_record_prerequisites_for_test(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Vec<Arc<WorthQueryOutputDemandKey>> {
        Self::release_record_prerequisites(self, key)
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn has_cached_ready_for_test(&self) -> bool {
        Self::has_cached_ready(self)
    }
}

impl DemandRecord {
    #[doc(hidden)]
    pub fn is_required_for_test(&self) -> bool {
        Self::is_required(self)
    }
}
