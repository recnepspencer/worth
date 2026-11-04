//! Sealed test doors over this module's private members.
use super::*;

impl Succession {
    #[doc(hidden)]
    pub fn new_for_test(predecessor: [u8; 32]) -> Self {
        Self::new(predecessor)
    }
}

impl super::super::DemandRegistryState {
    #[doc(hidden)]
    pub fn revive_replaced_ready_for_test(
        &mut self,
        key: &super::super::WorthQueryOutputDemandKey,
    ) -> bool {
        Self::revive_replaced_ready(self, key)
    }
}

impl Succession {
    #[doc(hidden)]
    pub fn predecessor_for_test(&self) -> [u8; 32] {
        Self::predecessor(self)
    }
}

impl Succession {
    #[doc(hidden)]
    pub fn predecessor_of_for_test(succession: &Option<Self>) -> Option<[u8; 32]> {
        Self::predecessor_of(succession)
    }
}
