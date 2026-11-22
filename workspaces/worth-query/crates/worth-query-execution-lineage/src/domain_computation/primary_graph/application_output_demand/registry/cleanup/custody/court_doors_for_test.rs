//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn drain_completed_custody_for_test(&mut self) {
        Self::drain_completed_custody(self)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn drain_next_foreign_custody_for_test(&mut self) {
        Self::drain_next_foreign_custody(self)
    }
}
