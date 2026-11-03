//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        &self.key
    }
}

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &WorthQueryOutputDemandRegistry {
        &self.owner
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<DemandRegistryState>> {
        &self.state
    }
}
