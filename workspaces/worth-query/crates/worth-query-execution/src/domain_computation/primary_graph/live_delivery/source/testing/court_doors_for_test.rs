//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn active_partition_count_for_test(&self) -> usize {
        Self::active_partition_count(self)
    }
}
