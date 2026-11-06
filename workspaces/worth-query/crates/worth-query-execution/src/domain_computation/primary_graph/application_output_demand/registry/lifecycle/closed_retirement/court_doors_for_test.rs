//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn follow_newest_for_test(
        &mut self,
        dependent: &WorthQueryOutputDemandKey,
        upstream: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandKey> {
        Self::follow_newest(self, dependent, upstream)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn retire_row_for_test(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        retired: &mut Vec<DemandRecord>,
    ) -> Vec<Arc<WorthQueryOutputDemandKey>> {
        Self::retire_row(self, key, retired)
    }
}
