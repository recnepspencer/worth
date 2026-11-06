//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn open_for_test(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryLiveSubscription, WorthQueryLiveSubscriptionCapacityExhausted> {
        Self::open(self, observation)
    }
}
