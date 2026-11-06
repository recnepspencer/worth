//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn note_closed_row_for_test(&mut self, key: &WorthQueryOutputDemandKey) {
        Self::note_closed_row(self, key)
    }
}

impl ClosedCleanupQueue {
    #[doc(hidden)]
    pub fn head_for_test(&self) -> Option<Arc<WorthQueryOutputDemandKey>> {
        Self::head(self)
    }
}

impl ClosedCleanupQueue {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl ClosedCleanupQueue {
    #[doc(hidden)]
    pub fn valid_order_for_test(&self) -> bool {
        Self::valid_order(self)
    }
}

impl ClosedCleanupLinks {
    #[doc(hidden)]
    pub fn queued_for_test(&self) -> bool {
        Self::queued(self)
    }
}
