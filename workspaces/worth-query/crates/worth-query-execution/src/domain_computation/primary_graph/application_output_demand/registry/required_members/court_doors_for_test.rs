//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn install_required_member_for_test(&mut self, prepared: Option<PreparedRequiredMember>) {
        Self::install_required_member(self, prepared)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn prepare_required_member_for_test(
        &self, key: &WorthQueryOutputDemandKey) -> Result<Option<PreparedRequiredMember>, WorthQueryOutputDemandDenial> {
        Self::prepare_required_member(self, key)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn remove_required_member_if_released_for_test(&mut self, key: &WorthQueryOutputDemandKey) {
        Self::remove_required_member_if_released(self, key)
    }
}
